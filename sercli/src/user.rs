use sqlx::{FromRow, postgres::PgRow};

use crate::{Entity, ID};

pub trait SercliUser: Entity + Clone + Send + Unpin + for<'r> FromRow<'r, PgRow> + 'static {
    fn id(&self) -> ID;
    fn password(&self) -> &str;
    fn login(&self) -> &str;
    fn login_field_name() -> &'static str;
}
