use model::REGISTER;
use sercli::server::Server;

use crate::requests::handle_register;

mod requests;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Server::new().add_request(&REGISTER, handle_register).start().await?;

    Ok(())
}
