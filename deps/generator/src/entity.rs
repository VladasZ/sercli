use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
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
    pub fn generate_file(&self, folder: &Path) -> Result<()> {
        let path = folder.join(self.file_name());

        let mut file = File::create_new(&path)?;

        file.write_all(self.to_code().as_bytes())?;

        Ok(())
    }

    pub(crate) fn to_code(&self) -> String {
        let name = &self.name;

        let mut fields = String::new();

        for field in &self.fields {
            fields.push_str(&field.to_code());
        }

        format!(
            r"
mod reflected {{
    pub use sercli::reflected::*;
}}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize, reflected::Reflected, sqlx::FromRow)]
pub struct {name} {{
{fields}}}
"
        )
    }

    fn file_name(&self) -> PathBuf {
        format!("{}.rs", self.name.to_snake_case()).into()
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
