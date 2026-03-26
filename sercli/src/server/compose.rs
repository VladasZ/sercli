use std::fs::read_to_string;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_yaml::from_str;

use crate::deps::utils::git_root;

#[derive(Debug, Deserialize)]
struct ComposeFile {
    services: Services,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Services {
    pg_rw: PgService,
}

#[derive(Debug, Deserialize)]
struct PgService {
    environment: Environment,
    ports:       Vec<String>,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct Environment {
    postgres_user:     String,
    postgres_password: String,
    postgres_db:       String,
}

pub fn connection_string_from_compose() -> Result<String> {
    let yaml = read_to_string(git_root()?.join("docker-compose.yml"))?;

    let compose: ComposeFile = from_str(&yaml).context("Invalid docker-compose.yml")?;

    let user = &compose.services.pg_rw.environment.postgres_user;
    let password = &compose.services.pg_rw.environment.postgres_password;
    let db = &compose.services.pg_rw.environment.postgres_db;

    let port_mapping = &compose.services.pg_rw.ports[0];
    let host_port = port_mapping.split(':').next().expect("Invalid port format");

    Ok(format!(
        "postgresql://{user}:{password}@localhost:{host_port}/{db}"
    ))
}

#[test]
fn test_connection_string() -> Result<()> {
    assert_eq!(
        connection_string_from_compose()?,
        "postgresql://sercli:sercli@localhost:54321/sercli_db"
    );

    Ok(())
}
