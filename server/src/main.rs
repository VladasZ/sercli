use anyhow::Result;
use sercli::start_server;

#[tokio::main]
async fn main() -> Result<()> {
    let handle = start_server();

    handle.await??;

    Ok(())
}
