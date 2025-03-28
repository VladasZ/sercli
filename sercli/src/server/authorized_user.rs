use std::fmt::Debug;

use anyhow::{Result, anyhow};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use derive_more::{Deref, DerefMut, From};
use sqlx::{PgPool, query};

use crate::{
    SercliUser,
    server::{AppError, access_token::AccessToken},
};

#[derive(Deref, DerefMut, From)]
pub struct AuthorizedUser<User: SercliUser> {
    #[deref]
    #[deref_mut]
    user: User,
    pool: PgPool,
}

impl<User: SercliUser> AuthorizedUser<User> {
    pub async fn revoke_all_tokens(&self) -> Result<()> {
        query("DELETE FROM token_storage WHERE user_id = $1")
            .bind(self.user.id())
            .execute(&self.pool)
            .await?;

        Ok(())
    }
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

        Ok(Self {
            user: AccessToken::check_token(token, &pool).await?,
            pool,
        })
    }
}
