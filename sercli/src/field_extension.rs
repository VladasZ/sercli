#![allow(async_fn_in_trait)]

use anyhow::Result;
use reflected::Field;
use sqlx::{Encode, PgPool, Postgres, Type};

use crate::Crud;

pub trait FieldExtension<T: Crud>: Sized {
    async fn one_where<'a, V: sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres> + Send + 'static>(
        &self,
        value: V,
        pool: &'a PgPool,
    ) -> Result<Option<T>>;
    async fn all_where<'a, V: sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres> + Send + 'static>(
        &self,
        value: V,
        pool: &'a PgPool,
    ) -> Result<Vec<T>>;
}

impl<T: Crud> FieldExtension<T> for Field<T> {
    async fn one_where<'a, V: Encode<'a, Postgres> + Type<Postgres> + Send + 'static>(
        &self,
        value: V,
        pool: &'a PgPool,
    ) -> Result<Option<T>> {
        T::get(pool).with(*self, value).one().await
    }
    async fn all_where<'a, V: sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres> + Send + 'static>(
        &self,
        value: V,
        pool: &'a PgPool,
    ) -> Result<Vec<T>> {
        T::get(pool).with(*self, value).all().await
    }
}
