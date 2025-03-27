use axum::{Json, extract::State};
use model::{User, Wallet};
use sercli::{
    Crud, Entity,
    server::{AppError, AuthorizedUser},
};
use sqlx::PgPool;

pub async fn create_wallet(
    user: AuthorizedUser<User>,
    db: State<PgPool>,
    wallet: Json<Wallet>,
) -> Result<Json<Vec<User>>, AppError> {
    let mut wallet = wallet.0;

    wallet.user_id = user.id;

    let wallet = wallet.insert(&db);

    Ok(Json(User::get_all(&db).await?))
}
