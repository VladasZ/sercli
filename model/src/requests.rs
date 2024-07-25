use sercli::Request;

use crate::User;

pub const REGISTER: Request<User, User> = Request::new("register");
