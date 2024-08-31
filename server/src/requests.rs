use axum::{http::HeaderMap, Json};
use model::User;
use sercli::server::{AppError, ToResponse};

type AuthSession = sercli::AuthSession<User>;

pub async fn handle_register(
    _: HeaderMap,
    session: AuthSession,
    user: Json<User>,
) -> Result<Json<User>, AppError> {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, age, name, password)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
        user.email,
        user.age,
        user.name,
        user.password,
    )
    .fetch_one(&session.backend.pg_pool)
    .await
    .to_response()
}

pub async fn handle_get_users(
    _: HeaderMap,
    session: AuthSession,
    _: Json<()>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users
        "#,
    )
    .fetch_all(&session.backend.pg_pool)
    .await?;

    Ok(Json(users))
}
