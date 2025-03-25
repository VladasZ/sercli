use anyhow::Result;
use sqlx::{Executor, PgPool, Postgres};

use crate::Entity;

#[allow(async_fn_in_trait)]
pub trait Crud: Sized {
    async fn get_all(pool: &PgPool) -> Result<Vec<Self>>;
    async fn create_table(pool: &PgPool) -> Result<()>;
    async fn drop_table(pool: &PgPool) -> Result<()>;
    async fn insert(self, pool: &PgPool) -> Result<Self>;
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

    async fn insert(self, pool: &PgPool) -> Result<Self> {
        let query = T::insert_query();

        let query = sqlx::query_as::<Postgres, T>(&query);

        let query = self.bind_to_sqlx_query(query);

        Ok(query.fetch_one(pool).await?)
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use reflected::Reflected;
    use sqlx::FromRow;

    use crate::{db::prepare_db, server::crud::Crud};

    #[derive(Debug, Clone, Default, PartialEq, Reflected, FromRow)]
    struct VaccinatedDog {
        name:   String,
        age:    i32,
        weight: f32,
    }

    #[tokio::test]
    async fn test() -> Result<()> {
        let pool = prepare_db().await?;

        let err = VaccinatedDog::get_all(&pool).await.expect_err("Get without table didn't fail");

        assert!(format!("{err}").contains("relation \"vaccinated_dogs\" does not exist"));

        VaccinatedDog::create_table(&pool).await?;

        assert_eq!(VaccinatedDog::get_all(&pool).await?, vec![]);

        let dog = VaccinatedDog {
            name:   "fedie".to_string(),
            age:    4234,
            weight: 42345454.43,
        };

        let inserted_dog = dog.clone().insert(&pool).await?;

        assert_eq!(inserted_dog, dog);

        let all = VaccinatedDog::get_all(&pool).await?;

        assert_eq!(all.first().unwrap(), &dog);

        VaccinatedDog::drop_table(&pool).await?;

        Ok(())
    }
}
