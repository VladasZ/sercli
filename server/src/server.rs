use model::{CREATE_WALLET, GET_USERS, GET_WALLETS, REGISTER};
use sercli::server::Server;

use crate::{
    user_requests::{get_users, handle_register},
    wallet_requests::{create_wallet, get_wallets},
};

pub fn make_server() -> Server {
    Server::new()
        .add_authorize_request(&REGISTER, handle_register)
        .add_authorized_request(&GET_USERS, get_users)
        .add_authorized_request(&CREATE_WALLET, create_wallet)
        .add_authorized_request(&GET_WALLETS, get_wallets)
}
