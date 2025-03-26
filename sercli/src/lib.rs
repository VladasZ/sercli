pub mod client;
pub mod db;
mod entity;
mod password;
pub mod server;
mod user;

pub use axum::{Json, extract::State, http::HeaderMap};
pub use entity::Entity;
pub use password::{check_password, hash_password};
pub use server::crud::Crud;
pub use user::SercliUser;

pub type ID = i32;
