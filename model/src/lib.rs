mod entities;
mod requests;
mod user;

use anyhow::Result;
pub use entities::*;
pub use requests::*;

#[ignore]
#[tokio::test]
async fn setup_db() -> Result<()> {
    use sercli::db::{generate_model, prepare_db};

    generate_model()?;
    prepare_db().await?;

    Ok(())
}

#[ignore]
#[tokio::test]
async fn wipe_db() -> Result<()> {
    sercli::db::wipe_db()
}
