use sercli::{SercliUser, server::AuthUser};

use crate::entities::User;

impl SercliUser for User {
    fn password(&self) -> &str {
        &self.password
    }

    fn login(&self) -> &str {
        &self.email
    }

    fn login_field_name() -> &'static str {
        "email"
    }
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}
