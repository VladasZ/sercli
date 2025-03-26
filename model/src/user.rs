use sercli::{ID, SercliUser};

use crate::entities::User;

impl SercliUser for User {
    fn id(&self) -> ID {
        self.id
    }

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
