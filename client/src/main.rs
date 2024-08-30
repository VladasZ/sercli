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
    use model::{User, GET_USERS, NON_EXISTING_ENDPOINT, REGISTER};
    use sercli::client::API;
    use server::make_server;
    use tokio::sync::oneshot::channel;

    #[tokio::test]
    async fn test_response_errors() -> anyhow::Result<()> {
        const USER_MAIL: &str = "user@gmail.com";

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

        let users = GET_USERS.send(()).await?;

        async fn register_peter() -> anyhow::Result<()> {
            REGISTER
                .send(User {
                    id:    0,
                    email: USER_MAIL.to_string(),
                    age:   20,
                    name:  "Peter".to_string(),
                })
                .await?;
            Ok(())
        }

        if !users.into_iter().any(|user| user.email == USER_MAIL) {
            register_peter().await?;
        }

        let error = register_peter().await.expect_err("Second register Peter should have failed");

        assert_eq!(
            format!("{error}"),
            "Something went wrong: User with such email already exists."
        );

        Ok(())
    }
}
