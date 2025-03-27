use anyhow::{Result, anyhow};
use sqlx::{Executor, PgPool, query};

pub struct DBStorage {}

impl DBStorage {
    pub async fn set(key: &str, value: &str, pool: &PgPool) -> Result<()> {
        Self::create_table(pool).await?;

        pool.execute(
            query(
                r"INSERT INTO key_value_storage (key, value)
              VALUES ($1, $2)
              ON CONFLICT (key)
              DO UPDATE SET value = EXCLUDED.value;
",
            )
            .bind(key)
            .bind(value),
        )
        .await
        .map_err(|e| anyhow!(e))?;

        Ok(())
    }

    pub async fn get(key: &str, pool: &PgPool) -> Result<Option<String>> {
        Self::create_table(pool).await?;

        let result: Option<(String,)> = sqlx::query_as("SELECT value FROM key_value_storage WHERE key = $1;")
            .bind(key)
            .fetch_optional(pool)
            .await?;

        Ok(result.map(|(value,)| value))
    }

    async fn create_table(pool: &PgPool) -> Result<()> {
        pool.execute(query(
            r"CREATE TABLE IF NOT EXISTS key_value_storage (
              key VARCHAR(255) PRIMARY KEY,
              value VARCHAR(255)
);",
        ))
        .await
        .map_err(|e| anyhow!(e))?;

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use anyhow::Result;
    use sqlx::Executor;

    use crate::{db::prepare_db, server::db_storage::DBStorage};

    #[tokio::test]
    async fn key_value_storage() -> Result<()> {
        let pool = prepare_db().await?;

        pool.execute(sqlx::query("DROP TABLE key_value_storage;")).await?;

        assert_eq!(DBStorage::get("sokol", &pool).await?, None);

        DBStorage::set("sokol", "sobaka", &pool).await?;

        assert_eq!(DBStorage::get("sokol", &pool).await?, Some("sobaka".to_string()));

        DBStorage::set("sokol", "boran", &pool).await?;

        assert_eq!(DBStorage::get("sokol", &pool).await?, Some("boran".to_string()));

        Ok(())
    }
}
