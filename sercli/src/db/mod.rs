use std::{env::set_var, time::Duration};

use anyhow::{Result, bail};
use generator::Generator;
use sercli_utils::git_root;
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use tain::Postgres;
use tokio::time::sleep;

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

pub fn generate_model() -> Result<()> {
    Generator::run()
}

pub async fn prepare_db() -> Result<PgPool> {
    Postgres::start_env()?;

    let pool = open_pool_when_available(&Postgres::connection_string()?).await?;

    let root = git_root()?;

    let migrations_path = root.join("model/migrations");

    let migrator = Migrator::new(migrations_path).await?;

    migrator.run(&pool).await?;

    unsafe { set_var("DATABASE_URL", Postgres::connection_string()?) };

    dbg!("Migrations: OK", Postgres::connection_string()?);

    Ok(pool)
}

pub fn wipe_db() -> Result<()> {
    Postgres::wipe_container_env()
}
