pub mod db;
mod deps;
mod password;
pub mod server;
mod user;

pub use axum::{Json, extract::State, http::HeaderMap};
pub use chrono::{Duration, NaiveDateTime as DateTime, Utc};
pub use deps::utils::git_root;
pub use password::{check_password, hash_password};
pub use server::{
    connection_string_from_compose,
    crud::{Crud, CrudRequest, FieldExtension},
    db_storage::DBStorage,
};
pub use user::SercliUser;

pub use crate::server::crud::Entity;

pub mod reflected {
    pub use reflected::{Field, RandomReflected, Reflected, ToReflectedString, ToReflectedVal, Type};
}

pub mod client {
    pub use netrun::rest::*;
}

pub mod axum {
    pub use axum::*;
}

pub use deps::generator::Migrations;
pub use rust_decimal::Decimal;

pub type ID = i64;
