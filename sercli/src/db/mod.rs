use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::{Result, bail};
use generator::Generator;
use sercli_utils::git_root;
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use tokio::time::sleep;

use crate::connection_string_from_compose;

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
    let conn = if let Ok(conn) = std::env::var("PG_CONNECTION_STRING") {
        conn
    } else {
        compose_up()?;

        connection_string_from_compose()?
    };

    dbg!(&conn);

    let pool = open_pool_when_available(&conn).await?;

    let root = if let Ok(root) = git_root() {
        root
    } else {
        PathBuf::from(std::env::var("ROOT_PATH")?)
    };

    let migrations_path = root.join("model/migrations");

    let migrator = Migrator::new(migrations_path).await?;

    migrator.run(&pool).await?;

    Ok(pool)
}

pub fn wipe_db() -> Result<()> {
    compose_down()?;
    Ok(())
}

fn compose_up() -> Result<()> {
    let status = Command::new("docker")
        .arg("compose")
        .arg("up")
        .arg("-d")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        println!("docker-compose up completed successfully.");
    } else {
        eprintln!("docker-compose up failed.");
    }

    Ok(())
}

fn compose_down() -> Result<()> {
    let status = Command::new("docker")
        .args(["compose", "down", "--volumes", "--remove-orphans"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        println!("docker-compose down completed successfully.");
    } else {
        eprintln!("docker-compose down failed.");
    }

    Ok(())
}
