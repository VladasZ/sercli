
#[derive(Debug, Default, PartialEq, reflected::Reflected)]
pub struct User {
   pub id: usize,
   pub email: String,
   pub age: i16,
   pub name: String,
   pub password: String,
}
