use std::ops::Deref;

use axum::{extract::State, http::HeaderMap, Json};
use model::User;
use sercli::server::{AppError, ToResponse};
use sqlx::PgPool;

pub async fn handle_register(
    _: HeaderMap,
    pool: State<PgPool>,
    user: Json<User>,
) -> Result<Json<User>, AppError> {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, age, name)
        VALUES ($1, $2, $3)
        RETURNING *
        "#,
        user.email,
        user.age,
        user.name
    )
    .fetch_one(pool.deref())
    .await
    .to_response()
}

pub async fn handle_get_users(
    _: HeaderMap,
    pool: State<PgPool>,
    _: Json<()>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users
        "#,
    )
    .fetch_all(pool.deref())
    .await?;

    Ok(Json(users))
}
