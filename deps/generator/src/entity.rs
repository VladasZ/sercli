use inflector::{Inflector, string::singularize::to_singular};
use sqlparser::ast::CreateTable;

use crate::field::Field;

#[derive(Debug, PartialEq)]
pub struct Entity {
    pub name:       String,
    pub table_name: String,
    pub fields:     Vec<Field>,
}

impl Entity {
    pub fn to_code(&self) -> String {
        let name = &self.name;

        let mut fields = String::new();

        for field in &self.fields {
            fields.push_str(&field.to_code());
        }

        format!(
            r"
#[derive(Debug, PartialEq, Reflected)]
pub struct {name} {{
{fields}}}
"
        )
    }
}

impl From<CreateTable> for Entity {
    fn from(value: CreateTable) -> Self {
        let table_name = format!("{}", value.name);
        let name = to_singular(&table_name).to_pascal_case();

        Self {
            name,
            table_name,
            fields: value.columns.into_iter().map(Into::into).collect(),
        }
    }
}
