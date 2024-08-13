use std::future::Future;

use anyhow::Result;
use axum::{extract::State, routing::get, Json, Router};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::PgPool;
use tokio::{net::TcpListener, runtime::Runtime, spawn};

use crate::{client::Request, db::prepare_db, server::AppError};

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
        runtime.block_on(async { self.start_internal().await })?;
        Ok(())
    }

    pub fn spawn(self) -> Result<()> {
        spawn(async {
            self.start_internal().await.expect("Failed to spawn server");
        });

        Ok(())
    }

    async fn start_internal(self) -> Result<()> {
        let listener = TcpListener::bind("0.0.0.0:8000").await?;
        axum::serve(listener, self.router.with_state(prepare_db().await?)).await?;

        Ok(())
    }
}
