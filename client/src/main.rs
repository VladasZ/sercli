use anyhow::Result;
use model::GET_USERS;
use sercli::client::API;
use server::make_server;
use tokio::sync::oneshot::channel;

#[tokio::main]
async fn main() -> Result<()> {
    let handle = make_server().spawn().await?;

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
    use model::{
        CREATE_WALLET, GET_USERS, GET_WALLETS, NON_EXISTING_ENDPOINT, REGISTER, User, Wallet, WalletType,
    };
    use sercli::{DateTime, Decimal, client::API};
    use server::make_server;
    use tokio::sync::oneshot::channel;

    #[tokio::test]
    async fn test_response_errors() -> Result<()> {
        static EMAIL: OnceLock<String> = OnceLock::new();

        make_server().spawn().await?;

        API::init("http://localhost:8000");

        let error = NON_EXISTING_ENDPOINT
            .await
            .expect_err("Non existing endpoint request should have failed");

        assert_eq!(
            format!("{error}"),
            "Endpoint http://localhost:8000/non_existing_endpoint not found. 404."
        );

        let datetime_str = "2025-03-29 14:30:45";
        let format = "%Y-%m-%d %H:%M:%S";

        let peter = User {
            id:       0,
            email:    EMAIL.get_or_init(|| FreeEmail().fake::<String>()).clone(),
            age:      20,
            password: "prostaf".to_string(),
            birthday: DateTime::parse_from_str(datetime_str, format)?.into(),
        };

        let (token, _user) = REGISTER.send(peter.clone()).await?;

        API::set_access_token(token);

        let error = REGISTER
            .send(peter.clone())
            .await
            .expect_err("Second register Peter should have failed");

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
                password: "prostaf".to_string(),
                birthday: peter.birthday,
            }
        );

        let wallet = Wallet {
            id:      0,
            user_id: 0,
            name:    "Money".to_string(),
            amount:  Decimal::from_str("1050.25")?,
            tp:      WalletType::Crypto,
        };

        let wallet = CREATE_WALLET.send(wallet).await?;

        assert!(wallet.id != 0 && wallet.user_id != 0);

        assert_eq!(GET_WALLETS.await?, vec![wallet]);

        Ok(())
    }
}
