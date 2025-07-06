#![cfg(test)]

use anyhow::Result;
use sercli::{Crud, db::prepare_db, reflected::Reflected};

use crate::User;

#[ignore]
#[tokio::test]
async fn generate_test_data() -> Result<()> {
    let pool = prepare_db().await?;

    dbg!(User::get_all(&pool).await?);

    if !User::any_exists(&pool).await? {
        let user1 = User::random().insert(&pool).await?;
        dbg!(&user1);
    }

    dbg!(User::get_all(&pool).await?);

    Ok(())
}
