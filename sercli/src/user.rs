use axum_login::AuthUser;
use sqlx::{FromRow, postgres::PgRow};

pub trait SercliUser: Clone + AuthUser<Id = i32> + Unpin + for<'r> FromRow<'r, PgRow> + 'static {
    fn password(&self) -> &str;
    fn login(&self) -> &str;
    fn login_field_name() -> &'static str;
}
