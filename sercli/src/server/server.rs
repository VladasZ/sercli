use std::future::Future;

use anyhow::Result;
use axum::{Json, Router, extract::State, handler::Handler, routing::get};
use serde::{Serialize, de::DeserializeOwned};
use sqlx::PgPool;
use tokio::{net::TcpListener, runtime::Runtime, spawn, sync::oneshot::Sender};

use crate::{
    SercliUser,
    client::Request,
    server::{AppError, AuthorizeRequest, ServerHandle, authorized_user::AuthorizedUser, prepare_db},
};

#[derive(Default)]
pub struct Server {
    router: Router<PgPool>,
}

impl Server {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn edit_router(self, edit: impl FnOnce(Router<PgPool>) -> Router<PgPool>) -> Self {
        Self {
            router: edit(self.router),
        }
    }

    pub fn add_request<
        In: Serialize + DeserializeOwned + Send + 'static,
        Out: Serialize + DeserializeOwned + Send + 'static,
        F: Future<Output = Result<Json<Out>, AppError>> + Sized + Send + 'static,
    >(
        mut self,
        request: &'static Request<In, Out>,
        method: fn(State<PgPool>, Json<In>) -> F,
    ) -> Self {
        self.router = self.router.route(&format!("/{}", request.name), get(method));
        self
    }

    pub fn add_authorize_request<
        In: Serialize + DeserializeOwned + Send + 'static,
        Out: Serialize + DeserializeOwned + Send + 'static,
        F: Future<Output = Result<Json<Out>, AppError>> + Sized + Send + 'static,
        User: SercliUser,
        T: 'static,
    >(
        mut self,
        request: &'static Request<In, Out>,
        method: fn(AuthorizeRequest<User>, State<PgPool>, _: Json<In>) -> F,
    ) -> Self
    where
        fn(AuthorizeRequest<User>, State<PgPool>, _: Json<In>) -> F: Handler<T, PgPool>,
    {
        self.router = self.router.route(&format!("/{}", request.name), get(method));
        self
    }

    pub fn add_authorized_request<
        In: Serialize + DeserializeOwned + Send + 'static,
        Out: Serialize + DeserializeOwned + Send + 'static,
        F: Future<Output = Result<Json<Out>, AppError>> + Sized + Send + 'static,
        User: SercliUser,
        T: 'static,
    >(
        mut self,
        request: &'static Request<In, Out>,
        method: fn(AuthorizedUser<User>, State<PgPool>, _: Json<In>) -> F,
    ) -> Self
    where
        fn(AuthorizedUser<User>, State<PgPool>, _: Json<In>) -> F: Handler<T, PgPool>,
    {
        self.router = self.router.route(&format!("/{}", request.name), get(method));
        self
    }

    pub fn start(self) -> Result<()> {
        let runtime = Runtime::new()?;
        runtime.block_on(async { self.start_internal(None).await })?;
        Ok(())
    }

    pub fn spawn(self, started: Option<Sender<ServerHandle>>) -> Result<()> {
        spawn(async {
            self.start_internal(started).await.expect("Failed to spawn server");
        });

        Ok(())
    }

    async fn start_internal(self, started: Option<Sender<ServerHandle>>) -> Result<()> {
        let listener = TcpListener::bind("0.0.0.0:8000").await?;

        let (handle, receiver) = ServerHandle::new();

        let server = axum::serve(
            listener,
            self.router.with_state(prepare_db().await?).into_make_service(),
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
