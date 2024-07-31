mod server;

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
