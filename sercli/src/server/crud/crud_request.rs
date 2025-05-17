use crate::Entity;
use reflected::Field;
use sqlx::Postgres;
use sqlx::postgres::PgArguments;
use sqlx::query::QueryAs;
use sqlx::{PgPool, query_as};
use std::marker::PhantomData;

pub struct CrudRequest<'a, T: Entity> {
    pool: &'a PgPool,
    binds: Vec<(
        Field<T>,
        Box<dyn FnOnce(QueryAs<'a, Postgres, T, PgArguments>) -> QueryAs<'a, Postgres, T, PgArguments>>,
    )>,
    _p: PhantomData<T>,
}

impl<'a, T: Entity> CrudRequest<'a, T> {
    pub(crate) fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            binds: vec![],
            _p: PhantomData,
        }
    }

    pub fn r#where<V>(mut self, field: Field<T>, value: V)
    where
        V: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres> + 'static,
    {
        self.binds.push((field, Box::new(|q| q.bind(value))));
    }
}
