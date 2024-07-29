use anyhow::Result;
use tokio::task::JoinHandle;

use crate::db::prepare_db;

pub mod client;
pub mod db;

async fn start_server_async() -> Result<()> {
    prepare_db().await?;

    Ok(())
}

pub fn start_server() -> JoinHandle<Result<()>> {
    let handle = if let Ok(runtime) = tokio::runtime::Handle::try_current() {
        runtime.spawn(start_server_async())
    } else {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.spawn(start_server_async())
    };

    handle
}
