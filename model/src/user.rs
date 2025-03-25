use sercli::SercliUser;

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
