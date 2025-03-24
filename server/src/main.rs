mod requests;
mod server;

use model::User;
use server::make_server;

fn main() -> anyhow::Result<()> {
    make_server().start::<User>()
}
