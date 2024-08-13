use std::future::Future;

use anyhow::Result;
use axum::{extract::State, routing::get, Json, Router};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::PgPool;
use tokio::{net::TcpListener, runtime::Runtime, spawn, sync::oneshot::Sender};

use crate::{
    client::Request,
    db::prepare_db,
    server::{AppError, ServerHandle},
};

#[derive(Default)]
pub struct Server {
    router: Router<PgPool>,
}

impl Server {
    pub fn new() -> Self {
        Self::default()
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

        let server = axum::serve(listener, self.router.with_state(prepare_db().await?))
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
