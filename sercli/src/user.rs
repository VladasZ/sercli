use sqlx::{FromRow, postgres::PgRow};

use crate::ID;

pub trait SercliUser: Clone + Send + Unpin + for<'r> FromRow<'r, PgRow> + 'static {
    fn id(&self) -> ID;
    fn password(&self) -> &str;
    fn login(&self) -> &str;
    fn login_field_name() -> &'static str;
}
