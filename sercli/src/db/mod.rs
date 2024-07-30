use std::path::PathBuf;

use sqlx::{migrate::Migrator, postgres::PgPoolOptions};
use tain::Postgres;

use crate::utils::git_root;

pub async fn prepare_db() -> anyhow::Result<()> {
    Postgres::start_env()?;

    let pool = PgPoolOptions::new().connect(&Postgres::connection_string()?).await?;

    let root = git_root()?;
    let root = root.to_string_lossy();

    let migrator = Migrator::new(PathBuf::try_from(format!("{root}/model/migrations"))?).await?;

    migrator.run(&pool).await?;

    dbg!("Migrations: OK");

    Ok(())
}
