#![allow(async_fn_in_trait)]

use anyhow::Result;
use reflected::Field;
use sqlx::{Encode, PgPool, Postgres, Type};

use crate::Crud;

pub trait FieldExtension<T: Crud>: Sized {
    async fn one_where<'a, V: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres>>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<Option<T>>;
    async fn all_where<'a, V: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres>>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<Vec<T>>;
}

impl<T: Crud> FieldExtension<T> for Field<T> {
    async fn one_where<'a, V: 'a + Encode<'a, Postgres> + Type<Postgres>>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<Option<T>> {
        T::one_where(*self, value, pool).await
    }
    async fn all_where<'a, V: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres>>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<Vec<T>> {
        T::all_where(*self, value, pool).await
    }
}
