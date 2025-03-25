use sqlx::{FromRow, postgres::PgRow};

pub trait SercliUser: Clone + Send + Unpin + for<'r> FromRow<'r, PgRow> + 'static {
    fn id(&self) -> i64;
    fn password(&self) -> &str;
    fn login(&self) -> &str;
    fn login_field_name() -> &'static str;
}
