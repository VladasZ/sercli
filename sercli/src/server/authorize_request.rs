use std::marker::PhantomData;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use fake::Fake;
use sqlx::PgPool;

use crate::{
    SercliUser,
    server::{AppError, TOKEN_STORAGE},
};

pub struct AuthorizeRequest<User: SercliUser> {
    _p: PhantomData<User>,
}

impl<User: SercliUser> AuthorizeRequest<User> {
    pub async fn generate_token(&self, user: &User) -> String {
        let token: String = 64.fake();

        TOKEN_STORAGE.lock().await.insert(token.clone(), user.id());

        token
    }
}

impl<S: Sync, User: SercliUser> FromRequestParts<S> for AuthorizeRequest<User>
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self { _p: PhantomData })
    }
}
