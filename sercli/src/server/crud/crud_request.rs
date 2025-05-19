use std::fmt::Write;

use anyhow::Result;
use reflected::Field;
use sqlx::{PgPool, Postgres, postgres::PgArguments, query::QueryAs, query_as};

use crate::Entity;

type ArgBind<'a, T> =
    Box<dyn FnOnce(QueryAs<'a, Postgres, T, PgArguments>) -> QueryAs<'a, Postgres, T, PgArguments> + Send>;

type Bind<'a, T> = (Field<T>, ArgBind<'a, T>);

pub struct CrudRequest<'pool, 'args, T: Entity> {
    pool:  &'pool PgPool,
    binds: Vec<Bind<'args, T>>,
    q_str: String,
}

impl<'pool, 'args, T: Entity + 'static> CrudRequest<'pool, 'args, T> {
    pub(crate) fn new(pool: &'pool PgPool) -> Self {
        Self {
            pool,
            binds: vec![],
            q_str: String::new(),
        }
    }

    pub fn with<V>(mut self, field: Field<T>, value: V) -> Self
    where V: sqlx::Encode<'args, Postgres> + sqlx::Type<Postgres> + Send + 'static {
        self.binds.push((field, Box::new(move |q| q.bind(value))));
        self
    }

    pub fn and<V>(self, field: Field<T>, value: V) -> Self
    where V: sqlx::Encode<'args, Postgres> + sqlx::Type<Postgres> + Send + 'static {
        self.with(field, value)
    }

    fn prepare_query(&mut self) -> Result<QueryAs<'args, Postgres, T, PgArguments>> {
        let query = prepare_string_query(&self.binds)?;

        self.q_str.clone_from(&query);

        // TODO:
        // I'm too lazy and stupid to figure out these lifetimes now
        // It leaks
        let query_str: &'static String = Box::leak(Box::new(query));

        let mut query = query_as(query_str);

        for (_, bind) in self.binds.drain(..) {
            query = bind(query);
        }

        Ok(query)
    }

    pub async fn one2(&'args mut self) -> Result<Option<T>> {
        let query = prepare_string_query(&self.binds)?;

        self.q_str.clone_from(&query);

        let mut query = query_as(&self.q_str);

        for (_, bind) in self.binds.drain(..) {
            query = bind(query);
        }

        Ok(query.fetch_optional(self.pool).await?)
    }

    pub async fn one(mut self) -> Result<Option<T>> {
        Ok(self.prepare_query()?.fetch_optional(self.pool).await?)
    }

    pub async fn all(mut self) -> Result<Vec<T>> {
        Ok(self.prepare_query()?.fetch_all(self.pool).await?)
    }
}

fn prepare_string_query<T: Entity>(binds: &[Bind<T>]) -> Result<String> {
    let mut query = format!("SELECT * FROM {} ", T::table_name());

    for (i, (field, _)) in binds.iter().enumerate() {
        if i == 0 {
            write!(query, "WHERE {} = ${} ", field.name, i + 1)?;
        } else {
            write!(query, "AND {} = ${} ", field.name, i + 1)?;
        }
    }

    Ok(query)
}
