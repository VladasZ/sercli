use axum::{Json, extract::State};
use model::User;
use sercli::{
    Crud,
    server::{AppError, AuthorizeRequest, AuthorizedUser},
};
use sqlx::PgPool;

pub async fn handle_register(
    request: AuthorizeRequest<User>,
    db: State<PgPool>,
    user: Json<User>,
) -> Result<Json<(String, User)>, AppError> {
    let user = user.0.insert(&db).await?;

    let token = request.generate_token(&user).await;

    Ok(Json((token, user)))
}

pub async fn get_users(
    _user: AuthorizedUser<User>,
    db: State<PgPool>,
    _: Json<()>,
) -> Result<Json<Vec<User>>, AppError> {
    Ok(Json(User::get_all(&db).await?))
}
