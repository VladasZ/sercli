use axum::{Json, extract::State};
use model::User;
use sercli::server::{AppError, AuthorizeRequest, AuthorizedUser};
use sqlx::PgPool;

pub async fn handle_register(
    request: AuthorizeRequest<User>,
    db: State<PgPool>,
    user: Json<User>,
) -> Result<Json<(String, User)>, AppError> {
    let user: User = sqlx::query_as!(
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
    .await?;

    let token = request.generate_token(&user).await;

    Ok(Json((token, user)))
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
