use axum::{Json, extract::State};
use model::User;
use sercli::server::{AppError, AuthorizedUser, ToResponse};
use sqlx::PgPool;

pub async fn register(db: State<PgPool>, user: Json<User>) -> Result<Json<User>, AppError> {
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
    .fetch_one(&*db)
    .await
    .to_response()
}

pub async fn get_users(
    _user: AuthorizedUser<User>,
    db: State<PgPool>,
    _: Json<()>,
) -> Result<Json<Vec<User>>, AppError> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT * FROM users
        "#,
    )
    .fetch_all(&*db)
    .await?;

    Ok(Json(users))
}
