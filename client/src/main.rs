use anyhow::Result;
use model::GET_USERS;
use sercli::client::API;
use server::make_server;
use tokio::sync::oneshot::channel;

#[tokio::main]
async fn main() -> Result<()> {
    let (se, rc) = channel();

    make_server().spawn(se.into())?;

    let handle = rc.await?;

    dbg!(&handle);

    API::init("http://localhost:8000");

    let users = GET_USERS.await?;

    dbg!(&users);

    Ok(())
}

#[cfg(test)]
mod test {
    use std::{str::FromStr, sync::OnceLock};

    use anyhow::Result;
    use fake::{Fake, faker::internet::en::FreeEmail};
    use model::{CREATE_WALLET, GET_USERS, GET_WALLETS, NON_EXISTING_ENDPOINT, REGISTER, User, Wallet};
    use sercli::{Decimal, client::API};
    use server::make_server;
    use tokio::sync::oneshot::channel;

    #[tokio::test]
    async fn test_response_errors() -> Result<()> {
        static EMAIL: OnceLock<String> = OnceLock::new();

        let (se, rc) = channel();

        make_server().spawn(se.into())?;

        let _handle = rc.await?;

        API::init("http://localhost:8000");

        let error = NON_EXISTING_ENDPOINT
            .await
            .expect_err("Non existing endpoint request should have failed");

        assert_eq!(
            format!("{error}"),
            "Endpoint http://localhost:8000/non_existing_endpoint not found. 404."
        );

        async fn register_peter() -> Result<(String, User)> {
            let (token, user) = REGISTER
                .send(User {
                    id:       0,
                    email:    EMAIL.get_or_init(|| FreeEmail().fake::<String>()).clone(),
                    age:      20,
                    name:     "Peter".to_string(),
                    password: "prostaf".to_string(),
                })
                .await?;
            Ok((token, user))
        }

        let (token, _user) = register_peter().await?;

        API::set_access_token(token);

        let error = register_peter().await.expect_err("Second register Peter should have failed");

        assert_eq!(
            format!("{error}"),
            "Something went wrong: error returned from database: duplicate key value violates unique \
             constraint \"users_email_key\""
        );

        let users = GET_USERS.await?;

        let Some(user) = users.into_iter().find(|user| user.email == *EMAIL.get().unwrap()) else {
            panic!("Created user not found");
        };

        assert_eq!(
            user,
            User {
                id:       user.id,
                email:    EMAIL.get_or_init(|| FreeEmail().fake::<String>()).clone(),
                age:      20,
                name:     "Peter".to_string(),
                password: "prostaf".to_string(),
            }
        );

        let wallet = Wallet {
            id:      0,
            user_id: 0,
            name:    "Money".to_string(),
            amount:  Decimal::from_str("1050.25")?,
        };

        let wallet = CREATE_WALLET.send(wallet).await?;

        assert!(wallet.id != 0 && wallet.user_id != 0);

        assert_eq!(GET_WALLETS.await?, vec![wallet]);

        Ok(())
    }
}
