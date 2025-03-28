use anyhow::{Result, anyhow};
use sqlx::{Executor, PgPool, query};

pub struct DBStorage {}

impl DBStorage {
    pub async fn set(key: &str, data: &[u8], pool: &PgPool) -> Result<()> {
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
            .bind(data),
        )
        .await
        .map_err(|e| anyhow!(e))?;

        Ok(())
    }

    pub async fn get(key: &str, pool: &PgPool) -> Result<Option<Vec<u8>>> {
        Self::create_table(pool).await?;

        let result: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT value FROM key_value_storage WHERE key = $1;")
                .bind(key)
                .fetch_optional(pool)
                .await?;

        Ok(result.map(|(value,)| value))
    }

    pub async fn del(key: &str, pool: &PgPool) -> Result<()> {
        sqlx::query("DELETE FROM key_value_storage WHERE key = $1;")
            .bind(key)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn set_str(key: &str, val: &str, pool: &PgPool) -> Result<()> {
        Self::set(key, val.as_bytes(), pool).await
    }

    pub async fn get_str(key: &str, pool: &PgPool) -> Result<Option<String>> {
        Ok(Self::get(key, pool).await?.map(|vec| String::from_utf8(vec).unwrap()))
    }

    async fn create_table(pool: &PgPool) -> Result<()> {
        pool.execute(query(
            r"CREATE TABLE IF NOT EXISTS key_value_storage (
              key VARCHAR(255) PRIMARY KEY,
              value BYTEA NOT NULL
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
    use fake::{Fake, Faker};

    use crate::{db::prepare_db, server::db_storage::DBStorage};

    #[tokio::test]
    async fn key_value_storage() -> Result<()> {
        let pool = prepare_db().await?;

        DBStorage::del("sokol", &pool).await?;
        DBStorage::del("buff", &pool).await?;

        assert_eq!(DBStorage::get("sokol", &pool).await?, None);

        DBStorage::set_str("sokol", "sobaka", &pool).await?;

        assert_eq!(
            DBStorage::get_str("sokol", &pool).await?,
            Some("sobaka".to_string())
        );

        DBStorage::set_str("sokol", "boran", &pool).await?;

        assert_eq!(
            DBStorage::get_str("sokol", &pool).await?,
            Some("boran".to_string())
        );

        assert_eq!(DBStorage::get("buff", &pool).await?, None);

        for _ in 0..100 {
            let buff: Vec<u8> = Faker.fake();

            DBStorage::set("buff", &buff, &pool).await?;

            assert_eq!(DBStorage::get("buff", &pool).await?, Some(buff));
        }

        Ok(())
    }
}
