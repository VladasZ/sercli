
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize, reflected::Reflected, sqlx::FromRow)]
pub struct User {
   pub id: i64,
   pub email: String,
   pub age: i16,
   pub name: String,
   pub password: String,
}
