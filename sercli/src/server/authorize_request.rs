use std::marker::PhantomData;

use anyhow::Result;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::PgPool;

use crate::{
    SercliUser,
    server::{AppError, access_token::AccessToken},
};

pub struct AuthorizeRequest<User: SercliUser> {
    pool: PgPool,
    _p:   PhantomData<User>,
}

impl<User: SercliUser> AuthorizeRequest<User> {
    pub async fn generate_token(&self, user: &User) -> Result<String> {
        AccessToken::generate_token(user, &self.pool).await
    }
}

impl<S: Sync, User: SercliUser> FromRequestParts<S> for AuthorizeRequest<User>
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        Ok(Self {
            pool,
            _p: PhantomData,
        })
    }
}
