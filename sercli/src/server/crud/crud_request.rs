use std::{fmt::Write, mem::transmute};

use anyhow::{Result, anyhow};
use reflected::Field;
use sqlx::{PgPool, Postgres, postgres::PgArguments, query::QueryAs, query_as};

use crate::Entity;

type ArgBind<T> = Box<
    dyn FnOnce(QueryAs<'static, Postgres, T, PgArguments>) -> QueryAs<'static, Postgres, T, PgArguments>
        + Send,
>;

type Bind<T> = (Field<T>, ArgBind<T>);

pub struct CrudRequest<'pool, T: Entity> {
    pool:  &'pool PgPool,
    binds: Vec<Bind<T>>,
    q_str: String,
}

impl<'pool, T: Entity> CrudRequest<'pool, T> {
    pub(crate) fn new(pool: &'pool PgPool) -> Self {
        Self {
            pool,
            binds: vec![],
            q_str: String::new(),
        }
    }

    pub fn with<V>(mut self, field: Field<T>, value: V) -> Self
    where V: sqlx::Encode<'static, Postgres> + sqlx::Type<Postgres> + Send + 'static {
        self.binds.push((field, Box::new(move |q| q.bind(value))));
        self
    }

    pub fn and<V>(self, field: Field<T>, value: V) -> Self
    where V: sqlx::Encode<'static, Postgres> + sqlx::Type<Postgres> + Send + 'static {
        self.with(field, value)
    }

    fn prepare_query(&mut self) -> Result<QueryAs<'static, Postgres, T, PgArguments>> {
        self.q_str = prepare_string_query(&self.binds)?;

        // TODO:
        // I'm too lazy and stupid to figure out these lifetimes now
        // Query never leaves the request and this string should live inside
        // `CrudRequest` until execution is done
        let query_str: &'static str = unsafe { transmute(self.q_str.as_str()) };

        let mut query = query_as(query_str);

        for (_, bind) in self.binds.drain(..) {
            query = bind(query);
        }

        Ok(query)
    }

    pub async fn one_opt(mut self) -> Result<Option<T>> {
        Ok(self.prepare_query()?.fetch_optional(self.pool).await?)
    }

    pub async fn one(self) -> Result<T> {
        let Some(val) = self.one_opt().await? else {
            return Err(anyhow!("{} not found", T::table_name()));
        };

        Ok(val)
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
