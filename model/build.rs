use anyhow::Result;
use sercli::db::prepare_db;

#[tokio::main]
pub async fn main() -> Result<()> {
    prepare_db().await?;
    Ok(())
}
