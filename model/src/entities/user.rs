
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
    pub fn wallets<'a>(&self, pool: &'a sqlx::PgPool) -> CrudRequest<'a, Wallet> {
        Wallet::get(pool).with(Wallet::USER_ID, self.id)
    }
}
