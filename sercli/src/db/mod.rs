use std::path::PathBuf;

use sqlx::{migrate::Migrator, postgres::PgPoolOptions};
use tain::Postgres;

pub async fn prepare_db() -> anyhow::Result<()> {
    Postgres::start_env()?;

    let pool = PgPoolOptions::new().connect(&Postgres::connection_string()?).await?;

    let migrator = Migrator::new(PathBuf::try_from("model/migrations")?).await?;

    migrator.run(&pool).await?;

    dbg!("Migrations: OK");

    Ok(())
}
