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
