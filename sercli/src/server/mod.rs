mod errors_handling;
mod handle;
mod server;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
pub use errors_handling::*;
pub use handle::*;
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
pub struct AppError(anyhow::Error);

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

impl<E> From<E> for AppError
where E: Into<anyhow::Error>
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
