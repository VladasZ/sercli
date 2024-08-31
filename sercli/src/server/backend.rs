use std::marker::PhantomData;

use anyhow::Result;
use async_trait::async_trait;
use axum_login::{AuthnBackend, UserId};
use password_auth::verify_password;
use sqlx::PgPool;

use crate::{db::prepare_db, server::AppError, SercliUser};

#[derive(Clone)]
pub struct Backend<User> {
    pub pg_pool: PgPool,
    _p:          PhantomData<fn() -> User>,
}

impl<User> Backend<User> {
    pub(crate) async fn new() -> Result<Self> {
        Ok(Self {
            pg_pool: prepare_db().await?,
            _p:      PhantomData,
        })
    }
}

#[async_trait]
impl<User: SercliUser> AuthnBackend for Backend<User> {
    type User = User;
    type Credentials = User;
    type Error = AppError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        let user: Self::User = sqlx::query_as(&format!(
            "select * from users where {} = ? ",
            User::login_field_name()
        ))
        .bind(creds.login())
        .fetch_one(&self.pg_pool)
        .await?;

        verify_password(user.password(), creds.password())?;

        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user: Self::User = sqlx::query_as("select * from users where id = ? ")
            .bind(user_id)
            .fetch_one(&self.pg_pool)
            .await?;

        Ok(Some(user))
    }
}
