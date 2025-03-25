use model::{GET_USERS, REGISTER};
use sercli::server::Server;

use crate::requests::{get_users, register};

pub fn make_server() -> Server {
    Server::new()
        .add_request(&REGISTER, register)
        .add_authorized_request(&GET_USERS, get_users)
}
