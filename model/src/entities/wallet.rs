mod reflected {
    pub use sercli::reflected::*;
}

#[derive(Debug, Default, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct fdsfsdfds {}

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
pub struct Wallet {
    pub id:      sercli::ID,
    pub user_id: i32,
    pub name:    String,
    // pub amount:  sercli::Decimal,
    // pub asdsfsd: fdsfsdfds,
}
