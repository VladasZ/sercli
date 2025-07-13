#![allow(async_fn_in_trait)]

use std::mem::transmute;

use anyhow::Result;
use reflected::Field;
use sqlx::{Encode, PgPool, Postgres, Type};

use crate::Crud;

pub trait FieldExtension<T: Crud>: Sized {
    async fn one_where<V: sqlx::Encode<'static, Postgres> + sqlx::Type<Postgres> + Send + 'static>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<Option<T>>;
    async fn all_where<V: sqlx::Encode<'static, Postgres> + sqlx::Type<Postgres> + Send + 'static>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<Vec<T>>;
    async fn delete_where<V: sqlx::Encode<'static, Postgres> + sqlx::Type<Postgres> + Send + 'static>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<()>;
}

impl<T: Crud> FieldExtension<T> for Field<T> {
    async fn one_where<V: Encode<'static, Postgres> + Type<Postgres> + Send + 'static>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<Option<T>> {
        T::get(pool).with(*self, value).one_opt().await
    }
    async fn all_where<V: sqlx::Encode<'static, Postgres> + sqlx::Type<Postgres> + Send + 'static>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<Vec<T>> {
        T::get(pool).with(*self, value).all().await
    }

    async fn delete_where<V: Encode<'static, Postgres> + Type<Postgres> + Send + 'static>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<()> {
        let query = format!("DELETE FROM {} WHERE {} = $1", T::table_name(), self.name);
        let query_str: &'static str = unsafe { transmute(query.as_str()) };

        sqlx::query(query_str).bind(value).execute(pool).await?;

        Ok(())
    }
}
