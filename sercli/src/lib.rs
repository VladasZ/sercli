pub mod client;
pub mod db;
mod field_extension;
mod password;
pub mod server;
mod user;

pub use axum::{Json, extract::State, http::HeaderMap};
pub use chrono::{Duration, NaiveDateTime as DateTime, Utc};
pub use field_extension::FieldExtension;
pub use password::{check_password, hash_password};
pub use server::{connection_string_from_compose, crud::Crud, db_storage::DBStorage};
pub use user::SercliUser;

pub use crate::server::crud::Entity;

pub mod reflected {
    pub use reflected::{Field, Reflected, ToReflectedString, ToReflectedVal, Type};
}

pub mod axum {
    pub use axum::*;
}

pub use rust_decimal::Decimal;

pub type ID = i32;
