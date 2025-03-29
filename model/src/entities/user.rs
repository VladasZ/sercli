
mod reflected {
    pub use sercli::reflected::*;
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize, reflected::Reflected, sqlx::FromRow)]
pub struct User {
    pub id: sercli::ID,
    pub email: String,
    pub password: String,
    pub age: i32,
    pub birthday: sercli::DateTime,
}
