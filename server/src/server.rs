use model::{GET_USERS, REGISTER};
use sercli::server::Server;

use crate::requests::{handle_get_users, handle_register};

pub fn make_server() -> Server {
    Server::new()
        .add_request(&REGISTER, handle_register)
        .add_request(&GET_USERS, handle_get_users)
}
