pub mod client;
pub mod db;
pub mod server;
mod user;
mod utils;

pub use axum::{extract::State, http::HeaderMap, Json};
pub use user::SercliUser;

use crate::server::Backend;

pub type AuthSession<User> = axum_login::AuthSession<Backend<User>>;
