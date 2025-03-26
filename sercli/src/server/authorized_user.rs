use std::{collections::BTreeMap, fmt::Debug};

use anyhow::anyhow;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use derive_more::{Deref, DerefMut, From};
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::{ID, SercliUser, server::AppError};

pub(crate) static TOKEN_STORAGE: Mutex<BTreeMap<String, ID>> = Mutex::const_new(BTreeMap::new());

#[derive(Deref, DerefMut, From)]
pub struct AuthorizedUser<User: SercliUser> {
    user: User,
}

impl<S: Sync, User: SercliUser + Debug> FromRequestParts<S> for AuthorizedUser<User>
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

        let Some(user_id) = TOKEN_STORAGE.lock().await.get(token).copied() else {
            return Err(anyhow!("Invalid authorization token").into());
        };

        let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1;")
            .bind(user_id)
            .fetch_one(&pool)
            .await?;

        Ok(user.into())
    }
}
