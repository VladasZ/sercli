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
    user: AuthorizedUser<User>,
    db: State<PgPool>,
    _: Json<()>,
) -> Result<Json<Vec<Wallet>>, AppError> {
    // Weird issue:
    // = note: this is a known limitation that will be removed in the future (see issue #100013 <https://github.com/rust-lang/rust/issues/100013> for more information)
    // let wallets = Wallet::FIELDS.user_id.all_where(user.id, &db).await?;

    let wallets = Wallet::get(&db).with(Wallet::USER_ID, user.id).all().await?;

    Ok(Json(wallets))
}
