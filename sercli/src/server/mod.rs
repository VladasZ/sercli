mod backend;
mod errors_handling;
mod handle;
mod header_map_utils;
mod server;

use std::fmt::{Debug, Display, Formatter};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
pub use axum_login::AuthUser;
pub use backend::*;
pub use errors_handling::*;
pub use handle::*;
pub use header_map_utils::*;
use password_auth::VerifyError;
pub use server::*;
use tokio::task::JoinHandle;

use crate::db::prepare_db;

async fn start_server_async() -> anyhow::Result<()> {
    prepare_db().await?;

    Ok(())
}

pub fn start_server() -> JoinHandle<anyhow::Result<()>> {
    if let Ok(runtime) = tokio::runtime::Handle::try_current() {
        runtime.spawn(start_server_async())
    } else {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.spawn(start_server_async())
    }
}

// Make our own error that wraps `anyhow::Error`.
#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for AppError {}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self(err)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self(value.into())
    }
}

impl From<VerifyError> for AppError {
    fn from(value: VerifyError) -> Self {
        Self(value.into())
    }
}
