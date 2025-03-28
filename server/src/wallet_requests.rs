use axum::{Json, extract::State};
use model::{User, Wallet};
use sercli::{
    Crud,
    server::{AppError, AuthorizedUser},
};
use sqlx::PgPool;

pub async fn create_wallet(
    user: AuthorizedUser<User>,
    db: State<PgPool>,
    wallet: Json<Wallet>,
) -> Result<Json<Wallet>, AppError> {
    let mut wallet = wallet.0;

    wallet.user_id = user.id;

    let wallet = wallet.insert(&db).await?;

    Ok(Json(wallet))
}

pub async fn get_wallets(
    _user: AuthorizedUser<User>,
    _db: State<PgPool>,
    _: Json<()>,
) -> Result<Json<Vec<Wallet>>, AppError> {
    //  Ok(Json(wallet))

    todo!()
}
