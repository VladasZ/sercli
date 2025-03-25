use model::{GET_USERS, REGISTER};
use sercli::server::Server;

use crate::requests::{get_users, handle_register};

pub fn make_server() -> Server {
    Server::new()
        .add_authorize_request(&REGISTER, handle_register)
        .add_authorized_request(&GET_USERS, get_users)
}
