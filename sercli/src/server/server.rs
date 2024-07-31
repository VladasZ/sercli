use std::future::Future;

use axum::{extract::State, routing::get, Json, Router};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::PgPool;
use tokio::net::TcpListener;

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

    pub async fn start(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
        axum::serve(listener, self.router.with_state(prepare_db().await?))
            .await
            .unwrap();

        Ok(())
    }
}
