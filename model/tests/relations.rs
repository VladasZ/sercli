use anyhow::Result;
use model::{User, Wallet};
use sercli::{Crud, db::prepare_db, reflected::RandomReflected};

#[tokio::test]
async fn user_wallets_relation() -> Result<()> {
    let pool = prepare_db("migrations").await?;

    let mut user = User::random();
    user = user.insert(&pool).await?;

    let mut wallet = Wallet::random();
    wallet.user_id = user.id;
    wallet = wallet.insert(&pool).await?;

    let wallets = user.wallets(&pool).await?;
    assert_eq!(wallets, vec![wallet]);

    Ok(())
}
