use anyhow::Result;
use axum::{extract::State, Json};
use model::{User, REGISTER};
use sercli::server::Server;
use sqlx::postgres::PgPool;

#[tokio::main]
async fn main() -> Result<()> {
    Server::new().add_request(&REGISTER, handle_register).start().await?;

    Ok(())
}

async fn handle_register(_pool: State<PgPool>, user: Json<User>) -> Json<User> {
    let response = User {
        id:   41242,
        age:  user.age * 2,
        name: format!("Haah lohh: {}", user.name),
    };
    Json(response)
}
