use sercli::client::Request;

use crate::User;

pub const REGISTER: Request<User, User> = Request::new("register");
