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

    let users = GET_USERS.send(()).await?;

    dbg!(&users);

    Ok(())
}

#[cfg(test)]
mod test {
    use std::sync::OnceLock;

    use anyhow::Result;
    use fake::{Fake, faker::internet::en::FreeEmail};
    use model::{GET_USERS, NON_EXISTING_ENDPOINT, REGISTER, User};
    use sercli::client::API;
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
            .send(())
            .await
            .expect_err("Non existing endpoint request should have failed");

        assert_eq!(
            format!("{error}"),
            "Endpoint http://localhost:8000/non_existing_endpoint not found. 404."
        );

        async fn register_peter() -> Result<String> {
            let (token, _user) = REGISTER
                .send(User {
                    id:       0,
                    email:    EMAIL.get_or_init(|| FreeEmail().fake::<String>()).clone(),
                    age:      20,
                    name:     "Peter".to_string(),
                    password: "prostaf".to_string(),
                })
                .await?;
            Ok(token)
        }

        let token = register_peter().await?;

        API::add_header("token", token);

        let error = register_peter().await.expect_err("Second register Peter should have failed");

        assert_eq!(
            format!("{error}"),
            "Something went wrong: error returned from database: duplicate key value violates unique \
             constraint \"users_email_key\""
        );

        let users = GET_USERS.send(()).await?;

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

        Ok(())
    }
}
