use anyhow::Result;
use sqlx::{Executor, PgPool};

use crate::Entity;

pub trait Crud: Sized {
    async fn get_all(pool: &PgPool) -> Result<Vec<Self>>;
    async fn create_table(pool: &PgPool) -> Result<()>;
    async fn drop_table(pool: &PgPool) -> Result<()>;
}

impl<T: Entity> Crud for T {
    async fn get_all(pool: &PgPool) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(&format!("SELECT * FROM {}", T::table_name()))
            .fetch_all(pool)
            .await?)
    }

    async fn create_table(pool: &PgPool) -> Result<()> {
        pool.execute(&*T::create_table_query()).await?;
        Ok(())
    }

    async fn drop_table(pool: &PgPool) -> Result<()> {
        pool.execute(&*format!("DROP TABLE {};", T::table_name())).await?;
        Ok(())
    }
}

//
// sqlx::query_as("select * from users where id = ? ")         .bind(user_id)
// .fetch_one(&self.pg_pool)
// .await?;

#[cfg(test)]
mod test {
    use anyhow::Result;
    use reflected::Reflected;
    use sqlx::FromRow;

    use crate::{db::prepare_db, server::crud::Crud};

    #[derive(Debug, Default, Reflected, FromRow)]
    struct VaccinatedDog {}

    #[tokio::test]
    async fn test() -> Result<()> {
        let pool = prepare_db().await?;

        let err = VaccinatedDog::get_all(&pool).await.expect_err("Get without table didn't fail");

        assert!(format!("{err}").contains("relation \"vaccinated_dogs\" does not exist"));

        VaccinatedDog::create_table(&pool).await?;

        VaccinatedDog::drop_table(&pool).await?;

        Ok(())
    }
}
