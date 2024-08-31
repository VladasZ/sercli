use reflected::Reflected;
use sercli::{server::AuthUser, SercliUser};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, Default, Clone, FromRow, Reflected)]
pub struct User {
    #[serde(default)]
    pub id:       i32,
    pub email:    String,
    pub age:      i16,
    pub name:     String,
    pub password: String,
}

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
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes()
    }
}
