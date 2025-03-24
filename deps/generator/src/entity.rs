use inflector::{Inflector, string::singularize::to_singular};
use sqlparser::ast::CreateTable;

#[derive(Debug, PartialEq)]
pub struct Entity {
    pub name:       String,
    pub table_name: String,
}

impl From<CreateTable> for Entity {
    fn from(value: CreateTable) -> Self {
        let table_name = format!("{}", value.name);
        let name = to_singular(&table_name).to_pascal_case();

        Self { name, table_name }
    }
}
