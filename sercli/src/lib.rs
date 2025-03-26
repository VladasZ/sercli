pub mod client;
pub mod db;
mod entity;
pub mod server;
mod user;

pub use axum::{Json, extract::State, http::HeaderMap};
pub use entity::Entity;
pub use server::crud::Crud;
pub use user::SercliUser;

pub type ID = i32;
