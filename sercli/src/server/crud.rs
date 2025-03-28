use anyhow::Result;
use reflected::Field;
use sqlx::{Executor, PgPool, Postgres, query, query_as};

use crate::{Entity, ID};

#[allow(async_fn_in_trait)]
pub trait Crud: Sized {
    async fn create_table(pool: &PgPool) -> Result<()>;
    async fn drop_table(pool: &PgPool) -> Result<()>;

    async fn insert(self, pool: &PgPool) -> Result<Self>;
    async fn get_all(pool: &PgPool) -> Result<Vec<Self>>;
    async fn with_id(id: i32, pool: &PgPool) -> Result<Self>;
    async fn with<'a, V: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres>>(
        field: Field<Self>,
        value: V,
        pool: &PgPool,
    ) -> Result<Option<Self>>;
    async fn delete(self, pool: &PgPool) -> Result<()>;
}

impl<T: Entity> Crud for T {
    async fn create_table(pool: &PgPool) -> Result<()> {
        pool.execute(&*T::create_table_query()).await?;
        Ok(())
    }

    async fn drop_table(pool: &PgPool) -> Result<()> {
        pool.execute(&*format!("DROP TABLE IF EXISTS {};", T::table_name())).await?;
        Ok(())
    }

    async fn insert(self, pool: &PgPool) -> Result<Self> {
        let query = T::insert_query();
        let query = sqlx::query_as::<Postgres, T>(&query);
        let query = self.bind_to_sqlx_query(query);

        Ok(query.fetch_one(pool).await?)
    }

    async fn get_all(pool: &PgPool) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(&format!("SELECT * FROM {}", T::table_name()))
            .fetch_all(pool)
            .await?)
    }

    async fn with_id(id: ID, pool: &PgPool) -> Result<Self> {
        Ok(
            sqlx::query_as(&format!("SELECT * FROM {} WHERE id = {id}", T::table_name()))
                .fetch_one(pool)
                .await?,
        )
    }

    async fn with<'a, V: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres>>(
        field: Field<Self>,
        value: V,
        pool: &PgPool,
    ) -> Result<Option<Self>> {
        let query = format!("SELECT * FROM {} WHERE {} = $1", Self::table_name(), field.name);

        // TODO:
        // I'm too lazy and stupid to figure out these lifetimes now
        let query_str: &'static String = Box::leak(Box::new(query));

        let result = query_as(query_str).bind(value).fetch_optional(pool).await?;

        let query: Box<String> = Box::new(String::from(query_str));

        drop(query);

        Ok(result)
    }

    async fn delete(self, pool: &PgPool) -> Result<()> {
        let id: ID = self.value_by_name("id").parse()?;

        query(&format!("DELETE FROM {} WHERE id = $1", T::table_name()))
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use reflected::Reflected;
    use sqlx::FromRow;

    use crate::{db::prepare_db, field_extension::FieldExtension, server::crud::Crud};

    #[derive(Debug, Clone, Default, PartialEq, Reflected, FromRow)]
    struct VaccinatedDog {
        id:     i32,
        name:   String,
        age:    i32,
        weight: f32,
    }

    #[tokio::test]
    async fn test() -> Result<()> {
        let pool = prepare_db().await?;

        VaccinatedDog::drop_table(&pool).await?;

        let err = VaccinatedDog::get_all(&pool).await.expect_err("Get without table didn't fail");

        assert!(format!("{err}").contains("relation \"vaccinated_dogs\" does not exist"));

        VaccinatedDog::create_table(&pool).await?;

        assert_eq!(VaccinatedDog::get_all(&pool).await?, vec![]);

        let dog = VaccinatedDog {
            id:     1,
            name:   "fedie".to_string(),
            age:    4234,
            weight: 42345454.43,
        };

        let inserted_dog = dog.clone().insert(&pool).await?;

        assert_eq!(inserted_dog, dog);

        let all = VaccinatedDog::get_all(&pool).await?;

        assert_eq!(all.first().unwrap(), &dog);

        assert_eq!(VaccinatedDog::with_id(1, &pool).await?, dog);

        assert_eq!(
            VaccinatedDog::with(VaccinatedDog::FIELDS.name, "fedie", &pool).await?,
            Some(dog.clone())
        );

        assert_eq!(
            VaccinatedDog::FIELDS.age.is(4234, &pool).await?,
            Some(dog.clone())
        );

        assert_eq!(VaccinatedDog::FIELDS.age.is(7564, &pool).await?, None);

        dog.delete(&pool).await?;

        assert_eq!(VaccinatedDog::get_all(&pool).await?, vec![]);

        VaccinatedDog::drop_table(&pool).await?;

        Ok(())
    }
}
