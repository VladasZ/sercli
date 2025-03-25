use std::collections::BTreeMap;

use anyhow::anyhow;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use derive_more::{Deref, DerefMut, From};
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::{SercliUser, server::AppError};

pub(crate) static TOKEN_STORAGE: Mutex<BTreeMap<String, i64>> = Mutex::const_new(BTreeMap::new());

#[derive(Deref, DerefMut, From)]
pub struct AuthorizedUser<User: SercliUser> {
    user: User,
}

impl<S: Sync, User: SercliUser> FromRequestParts<S> for AuthorizedUser<User>
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);

        let Some(token) = parts.headers.get("token") else {
            return Err(anyhow!("Authorized request must have 'token' header").into());
        };

        let token = token.to_str()?;

        let Some(user_id) = dbg!(TOKEN_STORAGE.lock().await).get(token).copied() else {
            dbg!(&token);
            return Err(anyhow!("Invalid authorization token").into());
        };

        let user: User = sqlx::query_as("SELECT * FROM users WHERE id = ? ")
            .bind(user_id)
            .fetch_one(&pool)
            .await?;

        Ok(user.into())
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
