mod server;
mod user_requests;

use server::make_server;

fn main() -> anyhow::Result<()> {
    make_server().start()
}
