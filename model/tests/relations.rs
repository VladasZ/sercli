use anyhow::Result;
use model::{User, Wallet};
use sercli::{Crud, db::prepare_db, reflected::RandomReflected};

async fn setup() -> Result<(sqlx::PgPool, User, Wallet, Wallet)> {
    let pool = prepare_db("migrations").await?;

    let mut user = User::random();
    user = user.insert(&pool).await?;

    let mut wallet1 = Wallet::random();
    wallet1.user_id = user.id;
    wallet1 = wallet1.insert(&pool).await?;

    let mut wallet2 = Wallet::random();
    wallet2.user_id = user.id;
    wallet2.name = format!("other_{}", wallet1.name);
    wallet2 = wallet2.insert(&pool).await?;

    Ok((pool, user, wallet1, wallet2))
}

#[tokio::test]
async fn all() -> Result<()> {
    let (pool, user, wallet1, wallet2) = setup().await?;

    let wallets = user.wallets(&pool).all().await?;
    assert_eq!(wallets, vec![wallet1, wallet2]);

    Ok(())
}

#[tokio::test]
async fn one() -> Result<()> {
    let (pool, user, wallet1, _wallet2) = setup().await?;

    let wallet = user.wallets(&pool).and(Wallet::ID, wallet1.id).one().await?;
    assert_eq!(wallet, wallet1);

    Ok(())
}

#[tokio::test]
async fn one_opt_found() -> Result<()> {
    let (pool, user, wallet1, _wallet2) = setup().await?;

    let wallet = user.wallets(&pool).and(Wallet::ID, wallet1.id).one_opt().await?;
    assert_eq!(wallet, Some(wallet1));

    Ok(())
}

#[tokio::test]
async fn one_opt_not_found() -> Result<()> {
    let (pool, user, _wallet1, _wallet2) = setup().await?;

    let wallet = user.wallets(&pool).and(Wallet::ID, -1).one_opt().await?;
    assert_eq!(wallet, None);

    Ok(())
}

#[tokio::test]
async fn with_additional_filter() -> Result<()> {
    let (pool, user, wallet1, _wallet2) = setup().await?;

    let wallets = user.wallets(&pool).and(Wallet::NAME, wallet1.name.clone()).all().await?;
    assert_eq!(wallets, vec![wallet1]);

    Ok(())
}
