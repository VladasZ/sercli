use anyhow::Result;
use model::{User, REGISTER};
use reflected::Reflected;
use sercli::client::API;

#[tokio::main]
async fn main() -> Result<()> {
    API::init("http://localhost");

    let user = User::random();

    dbg!(&user);

    let user = REGISTER.send(user).await?;

    dbg!(&user);

    Ok(())
}
