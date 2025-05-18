#![allow(async_fn_in_trait)]

use std::fmt::Debug;

use anyhow::Result;
use reflected::Field;
use sqlx::{Encode, PgPool, Postgres, Type};

use crate::Crud;

pub trait FieldExtension<T: Crud>: Sized {
    async fn one_where<
        'a,
        V: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres> + Send + Debug + 'static,
    >(
        &self,
        value: V,
        pool: &'a PgPool,
    ) -> Result<Option<T>>;
    async fn all_where<
        'a,
        V: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres> + Send + Debug + 'static,
    >(
        &self,
        value: V,
        pool: &'a PgPool,
    ) -> Result<Vec<T>>;
}

impl<T: Crud> FieldExtension<T> for Field<T> {
    async fn one_where<'a, V: 'a + Encode<'a, Postgres> + Type<Postgres> + Send + Debug + 'static>(
        &self,
        value: V,
        pool: &'a PgPool,
    ) -> Result<Option<T>> {
        T::get(pool).with(*self, value).one().await
    }
    async fn all_where<
        'a,
        V: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres> + Send + Debug + 'static,
    >(
        &self,
        value: V,
        pool: &'a PgPool,
    ) -> Result<Vec<T>> {
        T::get(pool).with(*self, value).all().await
    }
}
