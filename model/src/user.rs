use reflected::Reflected;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Reflected)]
pub struct User {
    #[serde(default)]
    pub id:   i32,
    pub age:  i16,
    pub name: String,
}
