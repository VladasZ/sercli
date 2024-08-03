use std::ops::Deref;

use axum::{extract::State, Json};
use model::User;
use sercli::server::AppError;
use sqlx::PgPool;

pub async fn handle_register(pool: State<PgPool>, user: Json<User>) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (age, name)
        VALUES ($1, $2)
        RETURNING *
        "#,
        user.age,
        user.name
    )
    .fetch_one(pool.deref())
    .await?;

    Ok(Json(user))
}