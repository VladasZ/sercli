use anyhow::Result;
use sqlx::{Executor, PgPool, Postgres, query};

use crate::{Entity, ID, server::crud::CrudRequest};

#[allow(async_fn_in_trait)]
pub trait Crud: Sized + Entity {
    async fn create_table(pool: &PgPool) -> Result<()>;
    async fn drop_table(pool: &PgPool) -> Result<()>;

    async fn insert(self, pool: &PgPool) -> Result<Self>;
    async fn get_all(pool: &PgPool) -> Result<Vec<Self>>;
    async fn with_id(id: i32, pool: &PgPool) -> Result<Self>;
    async fn delete(self, pool: &PgPool) -> Result<()>;

    fn get(pool: &PgPool) -> CrudRequest<Self>;
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

    async fn delete(self, pool: &PgPool) -> Result<()> {
        let id: ID = self.value_by_name("id").parse()?;

        query(&format!("DELETE FROM {} WHERE id = $1", T::table_name()))
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    fn get(pool: &PgPool) -> CrudRequest<Self> {
        CrudRequest::new(pool)
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use reflected::{Reflected, ToReflectedVal};
    use sqlx::FromRow;

    use crate::{db::prepare_db, field_extension::FieldExtension, server::crud::Crud};

    #[derive(
        strum::Display,
        strum::EnumString,
        serde::Serialize,
        serde::Deserialize,
        sqlx::Type,
        Copy,
        Clone,
        Default,
        PartialEq,
        Debug,
    )]
    #[sqlx(type_name = "wallet_type", rename_all = "lowercase")]
    pub enum WalletType {
        #[default]
        Fiat,
        Crypto,
    }

    impl ToReflectedVal<WalletType> for &str {
        fn to_reflected_val(&self) -> std::result::Result<WalletType, String> {
            use std::str::FromStr;
            Ok(WalletType::from_str(self).unwrap())
        }
    }

    #[derive(Debug, Clone, Default, PartialEq, Reflected, FromRow)]
    struct VaccinatedDog {
        id:     i32,
        name:   String,
        age:    i32,
        weight: f32,
        tp:     WalletType,
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
            weight: 42.43,
            tp:     WalletType::Crypto,
        };

        let inserted_dog = dog.clone().insert(&pool).await?;

        assert_eq!(inserted_dog, dog);

        let no_dog = VaccinatedDog::get(&pool)
            .with(VaccinatedDog::NAME, "bon")
            .and(VaccinatedDog::AGE, 150)
            .and(VaccinatedDog::WEIGHT, 150.5)
            .one()
            .await?;

        assert_eq!(no_dog, None);

        let found_dog = VaccinatedDog::get(&pool)
            .with(VaccinatedDog::NAME, "fedie")
            .and(VaccinatedDog::AGE, 4234)
            .and(VaccinatedDog::ID, inserted_dog.id)
            // .and(VaccinatedDog::WEIGHT, 42.43) TODO: why no found by float?
            .one()
            .await?;

        assert_eq!(found_dog, Some(inserted_dog));

        let all = VaccinatedDog::get_all(&pool).await?;

        assert_eq!(all.first().unwrap(), &dog);

        assert_eq!(VaccinatedDog::with_id(1, &pool).await?, dog);

        assert_eq!(
            VaccinatedDog::get(&pool).with(VaccinatedDog::NAME, "fedie").one().await?,
            Some(dog.clone())
        );

        assert_eq!(
            VaccinatedDog::get(&pool).with(VaccinatedDog::NAME, "fedie").all().await?,
            vec![dog.clone()]
        );

        assert_eq!(
            VaccinatedDog::AGE.one_where(4234, &pool).await?,
            Some(dog.clone())
        );

        assert_eq!(
            VaccinatedDog::AGE.all_where(4234, &pool).await?,
            vec![dog.clone()]
        );

        assert_eq!(VaccinatedDog::AGE.one_where(7564, &pool).await?, None);
        assert_eq!(VaccinatedDog::AGE.all_where(7564, &pool).await?, vec![]);

        dog.delete(&pool).await?;

        assert_eq!(VaccinatedDog::get_all(&pool).await?, vec![]);

        VaccinatedDog::drop_table(&pool).await?;

        Ok(())
    }
}
