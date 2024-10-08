use sercli::client::Request;

use crate::User;

pub const REGISTER: Request<User, User> = Request::new("register");
pub const GET_USERS: Request<(), Vec<User>> = Request::new("get_users");
pub const NON_EXISTING_ENDPOINT: Request<(), ()> = Request::new("non_existing_endpoint");
