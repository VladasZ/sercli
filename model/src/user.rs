use reflected::Reflected;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Reflected)]
pub struct User {
    #[serde(default)]
    pub id:    i32,
    pub email: String,
    pub age:   i16,
    pub name:  String,
}
