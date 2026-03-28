use std::{
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::{Context, Result, bail};
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use tokio::time::sleep;

use crate::{
    connection_string_from_compose,
    deps::{
        generator::Generator,
        utils::{compose_path, migrations_path},
    },
};

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

async fn reset_if_tables_missing(pool: &PgPool) -> Result<()> {
    let migration_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name = \
         '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await?;

    if migration_count == 0 {
        return Ok(());
    }

    let applied: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = true")
        .fetch_one(pool)
        .await?;

    if applied == 0 {
        return Ok(());
    }

    let user_tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name != \
         '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await?;

    if user_tables == 0 {
        sqlx::query("DELETE FROM _sqlx_migrations").execute(pool).await?;
    }

    Ok(())
}

pub fn generate_model() -> Result<()> {
    Generator::run(&migrations_path()?)
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

    let migrations = migrations_path()?;

    let migrator = Migrator::new(migrations.as_path())
        .await
        .inspect_err(|err| {
            dbg!(err);
            dbg!(std::env::current_dir().unwrap());
        })
        .with_context(|| format!("Creating migrator with path: {}", migrations.display()))?;

    reset_if_tables_missing(&pool).await?;

    migrator.run(&pool).await?;

    Ok(pool)
}

pub fn stop_containers() -> Result<()> {
    compose_down()?;
    Ok(())
}

fn compose_up() -> Result<()> {
    let status = Command::new("docker")
        .arg("compose")
        .args(["-f", &compose_path()?.to_string_lossy()])
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
        .arg("compose")
        .args(["-f", &compose_path()?.to_string_lossy()])
        .args(["down", "--volumes", "--remove-orphans"])
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
