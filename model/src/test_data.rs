#![cfg(test)]

use anyhow::Result;
use rand::prelude::IndexedRandom;
use sercli::{Crud, db::prepare_db, reflected::RandomReflected};

use crate::{Dog, User, Wallet};

#[ignore]
#[tokio::test]
async fn generate_test_data() -> Result<()> {
    let pool = prepare_db().await?;

    dbg!(User::get_all(&pool).await?);

    if !User::any_exists(&pool).await? {
        for _ in 0..10 {
            User::random().insert(&pool).await?;
        }
    }

    if !Wallet::any_exists(&pool).await? {
        let users = User::get_all(&pool).await?;

        for _ in 0..5 {
            let user = users.choose(&mut rand::rng()).unwrap();
            let mut wallet = Wallet::random();
            wallet.user_id = user.id;
            wallet.insert(&pool).await?;
        }
    }

    if !Dog::any_exists(&pool).await? {
        let users = User::get_all(&pool).await?;

        for _ in 0..5 {
            let user = users.choose(&mut rand::rng()).unwrap();
            let mut wallet = Dog::random();
            wallet.user_id = user.id;
            wallet.insert(&pool).await?;
        }

        Dog::random().insert(&pool).await?;
        Dog::random().insert(&pool).await?;
    }

    dbg!(User::get_all(&pool).await?);
    dbg!(Wallet::get_all(&pool).await?);

    Ok(())
}
