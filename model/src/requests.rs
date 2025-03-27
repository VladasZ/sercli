use sercli::{ID, client::Request};

use crate::{Wallet, entities::User};

pub const REGISTER: Request<User, (String, User)> = Request::new("register");
pub const GET_USERS: Request<(), Vec<User>> = Request::new("get_users");

pub const CREATE_WALLET: Request<Wallet, Wallet> = Request::new("create_wallet");
pub const GET_WALLETS: Request<ID, Vec<Wallet>> = Request::new("get_wallets");

pub const NON_EXISTING_ENDPOINT: Request<(), ()> = Request::new("non_existing_endpoint");
