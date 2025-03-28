mod entities;
mod requests;
mod user;

pub use entities::*;
pub use requests::*;

#[ignore]
#[tokio::test]
async fn setup_db() -> anyhow::Result<()> {
    use sercli::db::{generate_model, prepare_db};

    generate_model()?;
    prepare_db().await?;

    Ok(())
}
