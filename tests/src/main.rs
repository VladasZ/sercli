
use sercli::db::prepare_db;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {

    prepare_db().await?;
    Ok(())
}
