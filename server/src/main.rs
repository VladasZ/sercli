mod requests;
mod server;

use server::make_server;

fn main() -> anyhow::Result<()> {
    make_server().start()
}
