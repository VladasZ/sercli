
#![allow(dead_code)]
#[allow(unused_imports)]
#[allow(clippy::wildcard_imports)]
use sercli::*;
use crate::Wallet;

mod reflected {
    pub use sercli::reflected::*;
}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    reflected::Reflected,
    sqlx::FromRow,
)]
pub struct User {
    pub id: ID,
    pub email: String,
    pub password: String,
    pub age: i32,
    pub birthday: Option<DateTime>,
    pub is_bot: Option<bool>,
}
impl User {
    pub async fn wallets(&self, pool: &sqlx::PgPool) -> anyhow::Result<Vec<Wallet>> {
        Ok(sqlx::query_as("SELECT * FROM wallets WHERE user_id = $1")
            .bind(self.id)
            .fetch_all(pool)
            .await?)
    }
}
