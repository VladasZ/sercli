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

// async fn authenticate(&self, creds: Self::Credentials) ->
// Result<Option<Self::User>, Self::Error> {     let user: Self::User =
// sqlx::query_as(&format!(         "select * from users where {} = ? ",
//         User::login_field_name()
//     ))
//         .bind(creds.login())
//         .fetch_one(&self.pg_pool)
//         .await?;
//
//     verify_password(user.password(), creds.password())?;
//
//     Ok(Some(user))
// }
//
// async fn get_user(&self, user_id: &UserId<Self>) ->
// Result<Option<Self::User>, Self::Error> {     let user: Self::User =
// sqlx::query_as("select * from users where id = ? ")         .bind(user_id)
//         .fetch_one(&self.pg_pool)
//         .await?;
//
//     Ok(Some(user))
// }
