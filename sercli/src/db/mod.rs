pub mod migration;

use std::{env::set_var, path::PathBuf, time::Duration};

use anyhow::{bail, Result};
use sqlx::{migrate::Migrator, postgres::PgPoolOptions, PgPool};
use tain::Postgres;
use tokio::time::sleep;

use crate::utils::git_root;

async fn open_pool_when_available(url: &str) -> Result<PgPool> {
    let mut pool: sqlx::Result<PgPool>;
    let mut retry_counter = 0;

    loop {
        pool = PgPoolOptions::new().connect(url).await;

        if let Ok(pool) = pool {
            return Ok(pool);
        }

        sleep(Duration::from_secs_f32(0.1)).await;

        retry_counter += 1;
        if retry_counter > 100 {
            bail!("Connection to PG pool reached retry limit of 100. Last result: {pool:?}");
        }
    }
}

pub async fn prepare_db() -> Result<PgPool> {

    dbg!("Forgol??");

    println!("cargo:rerun-if-changed=build.rs");

    Postgres::start_env()?;

    dbg!("Forgol??");

    let pool = open_pool_when_available(&Postgres::connection_string()?).await?;

    dbg!("Forgol??");


    let root = git_root()?;
    let root = root.to_string_lossy();

    let migrator = Migrator::new(PathBuf::from(format!("{root}/model/migrations"))).await?;

    migrator.run(&pool).await?;

    unsafe { set_var("DATABASE_URL", Postgres::connection_string()?) };

    dbg!("Migrations: OK");

    Ok(pool)
}
