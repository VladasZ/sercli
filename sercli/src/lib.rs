use anyhow::Result;
use tokio::task::JoinHandle;

use crate::db::prepare_db;

pub mod client;
pub mod db;
pub mod server;
mod utils;

async fn start_server_async() -> Result<()> {
    prepare_db().await?;

    Ok(())
}

pub fn start_server() -> JoinHandle<Result<()>> {
    if let Ok(runtime) = tokio::runtime::Handle::try_current() {
        runtime.spawn(start_server_async())
    } else {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.spawn(start_server_async())
    }
}
