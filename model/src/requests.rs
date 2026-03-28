use sercli::client::{Request, RestAPI};

use crate::{Wallet, entities::User};

pub static API: RestAPI = RestAPI::new("http://localhost:8001");

pub const REGISTER: Request<User, (String, User)> = API.request("register");
pub const GET_USERS: Request<(), Vec<User>> = API.request("get_users");

pub const CREATE_WALLET: Request<Wallet, Wallet> = API.request("create_wallet");
pub const GET_WALLETS: Request<(), Vec<Wallet>> = API.request("get_wallets");

pub const NON_EXISTING_ENDPOINT: Request<(), ()> = API.request("non_existing_endpoint");
