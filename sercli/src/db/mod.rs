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
        utils::{compose_path, migrations_path, target_dir},
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

    println!("Connecting to {}", hide_password(&conn));

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

/// The connection string carries the db password, and `prepare_db` runs on
/// every boot, so printing it whole wrote the password into the logs of every
/// service built on this crate. The host and the db name are the part worth
/// seeing, so only the password is replaced.
///
/// Written by hand rather than with a url crate because the string is not
/// always a valid url: `connection_string_from_compose` builds it from the
/// compose file, and a malformed one still has to print something instead of
/// failing the boot.
fn hide_password(conn: &str) -> String {
    let Some((scheme, rest)) = conn.split_once("://") else {
        return conn.to_string();
    };

    // The password ends at the last '@' of the authority, since it may itself
    // contain one. Everything from the first '/' on is the path.
    let authority_len = rest.find('/').unwrap_or(rest.len());
    let (authority, path) = rest.split_at(authority_len);

    let Some((userinfo, host)) = authority.rsplit_once('@') else {
        return conn.to_string();
    };

    let Some((user, _password)) = userinfo.split_once(':') else {
        return conn.to_string();
    };

    format!("{scheme}://{user}:***@{host}{path}")
}

pub fn stop_containers() -> Result<()> {
    compose_down()?;
    Ok(())
}

fn compose_up() -> Result<()> {
    let mut cmd = Command::new("docker");
    cmd.arg("compose")
        .args(["-f", &compose_path()?.to_string_lossy()])
        .arg("up")
        .arg("-d")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if let Some(dir) = target_dir()? {
        cmd.env("TARGET_DIR", dir);
    }

    let status = cmd.status()?;

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

#[cfg(test)]
mod tests {
    use super::hide_password;

    /// The regression: a real connection string used to be printed whole on
    /// every boot. Only the password goes, the host and db stay readable.
    #[test]
    fn test_password_is_hidden() {
        assert_eq!(
            hide_password("postgresql://petuh:s3cret@pg-rw:5432/petuh_db"),
            "postgresql://petuh:***@pg-rw:5432/petuh_db"
        );
        assert_eq!(
            hide_password("postgres://user:p@ss@localhost:5432/db"),
            "postgres://user:***@localhost:5432/db"
        );
    }

    /// A string with no password to hide comes back untouched. It must never
    /// fail the boot, so anything unparseable is printed as it is.
    #[test]
    fn test_strings_without_a_password_pass_through() {
        for conn in [
            "postgresql://pg-rw:5432/petuh_db",
            "postgresql://petuh@pg-rw:5432/petuh_db",
            "not a url at all",
        ] {
            assert_eq!(hide_password(conn), conn);
        }
    }
}
