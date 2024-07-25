use reflected::Reflected;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Reflected)]
pub struct User {
    pub id:   u32,
    pub age:  u32,
    pub name: String,
}
