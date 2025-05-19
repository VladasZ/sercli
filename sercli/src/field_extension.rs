#![allow(async_fn_in_trait)]

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
}

impl<T: Crud> FieldExtension<T> for Field<T> {
    async fn one_where<V: Encode<'static, Postgres> + Type<Postgres> + Send + 'static>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<Option<T>> {
        T::get(pool).with(*self, value).one().await
    }
    async fn all_where<V: sqlx::Encode<'static, Postgres> + sqlx::Type<Postgres> + Send + 'static>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<Vec<T>> {
        T::get(pool).with(*self, value).all().await
    }
}
