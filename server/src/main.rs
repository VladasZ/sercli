use anyhow::anyhow;
use axum::{extract::State, Json};
use model::{User, REGISTER};
use sercli::server::{AppError, Server};
use sqlx::postgres::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Server::new().add_request(&REGISTER, handle_register).start().await?;

    Ok(())
}

async fn handle_register(_pool: State<PgPool>, user: Json<User>) -> Result<Json<User>, AppError> {
    if user.age == 555 {
        return Err(anyhow!("aaaaa").into());
    }

    let response = User {
        id:   41242,
        age:  user.age * 2,
        name: format!("Haah lohh: {}", user.name),
    };
    Ok(Json(response))
}
