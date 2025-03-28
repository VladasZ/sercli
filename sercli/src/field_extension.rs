#![allow(async_fn_in_trait)]

use anyhow::Result;
use reflected::Field;
use sqlx::{Encode, PgPool, Postgres, Type};

use crate::Crud;

pub trait FieldExtension<T: Crud>: Sized {
    async fn is<'a, V: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres>>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<Option<T>>;
}

impl<T: Crud> FieldExtension<T> for Field<T> {
    async fn is<'a, V: 'a + Encode<'a, Postgres> + Type<Postgres>>(
        &self,
        value: V,
        pool: &PgPool,
    ) -> Result<Option<T>> {
        T::with(*self, value, pool).await
    }
}
