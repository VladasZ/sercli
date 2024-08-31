use std::future::Future;

use anyhow::Result;
use axum::{http::HeaderMap, routing::get, Json, Router};
use axum_login::{
    predicate_required,
    tower_sessions::{MemoryStore, SessionManagerLayer},
    AuthManagerLayerBuilder, AuthSession,
};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{net::TcpListener, runtime::Runtime, spawn, sync::oneshot::Sender};

use crate::{
    client::Request,
    server::{backend::Backend, AppError, ServerHandle},
    SercliUser,
};

#[derive(Default)]
pub struct Server {
    router: Router<()>,
}

impl Server {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_request<
        In: Serialize + DeserializeOwned + Send + 'static,
        Out: Serialize + DeserializeOwned + Send + 'static,
        F: Future<Output = Result<Json<Out>, AppError>> + Sized + Send + 'static,
        User: SercliUser,
    >(
        mut self,
        request: &'static Request<In, Out>,
        method: fn(HeaderMap, AuthSession<Backend<User>>, Json<In>) -> F,
    ) -> Self {
        self.router = self.router.route(&format!("/{}", request.name), get(method));
        self
    }

    pub fn start<User: SercliUser>(self) -> Result<()> {
        let runtime = Runtime::new()?;
        runtime.block_on(async { self.start_internal::<User>(None).await })?;
        Ok(())
    }

    pub fn spawn<User: SercliUser>(self, started: Option<Sender<ServerHandle>>) -> Result<()> {
        spawn(async {
            self.start_internal::<User>(started).await.expect("Failed to spawn server");
        });

        Ok(())
    }

    async fn start_internal<User: SercliUser>(self, started: Option<Sender<ServerHandle>>) -> Result<()> {
        // Session layer.
        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store);

        // Auth service.
        let backend = Backend::<User>::new().await?;
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let listener = TcpListener::bind("0.0.0.0:8000").await?;

        let (handle, receiver) = ServerHandle::new();

        dbg!(&self.router);

        let server = axum::serve(
            listener,
            self.router
                .route_layer(predicate_required!(
                    is_authenticated::<User>,
                    login_url = "/login",
                    redirect_field = "next"
                ))
                // ^
                // .route_layer(login_required!(Backend<User>, login_url = "/login"))
                // .route("/login", post(todo!()))
                // .route("/login", get(todo!()))
                .layer(auth_layer) // .with_state(prepare_db().await?),
                .into_make_service(),
        )
        .with_graceful_shutdown(receiver);

        if let Some(started) = started {
            tokio::join!(server, async {
                started.send(handle).unwrap();
            })
            .0?;
        } else {
            server.await?;
        }

        Ok(())
    }
}

async fn is_authenticated<User: SercliUser>(auth_session: AuthSession<Backend<User>>) -> bool {
    auth_session.user.is_some()
}
