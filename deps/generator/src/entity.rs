use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use inflector::{Inflector, string::singularize::to_singular};
use sqlparser::ast::{AlterTableOperation, CreateTable, ObjectName, ObjectNamePart};

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

    pub(crate) fn process_alter_table_operations(&mut self, operations: Vec<AlterTableOperation>) {
        for op in operations {
            self.process_alter_table_operation(op);
        }
    }

    fn process_alter_table_operation(&mut self, operation: AlterTableOperation) {
        match operation {
            AlterTableOperation::AddColumn {
                column_keyword: _,
                if_not_exists: _,
                column_def,
                column_position: _,
            } => self.fields.push(column_def.into()),
            AlterTableOperation::AddConstraint(_) => {}
            _ => unimplemented!("Unsipported alter table operation: {operation}"),
        }
    }

    pub(crate) fn to_code(&self) -> String {
        let name = &self.name;

        let mut fields = String::new();

        for field in &self.fields {
            fields.push_str(&field.to_code());
        }

        format!(
            r"
#[allow(clippy::wildcard_imports)]
use sercli::*;

mod reflected {{
    pub use sercli::reflected::*;
}}

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    reflected::Reflected,
    sqlx::FromRow,
)]
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
        let table_name = format!("{}", value.name).replace('"', "");

        Self {
            name: name_to_table_name(&table_name),
            table_name,
            fields: value.columns.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ObjectName> for Entity {
    fn from(value: ObjectName) -> Self {
        if value.0.len() != 1 {
            panic!("Check what is there");
        }

        let part = value.0.first().unwrap();

        let table_name = match part {
            ObjectNamePart::Identifier(ident) => ident.to_string().replace('"', ""),
        };

        Self {
            name: name_to_table_name(&table_name),
            table_name,
            fields: vec![],
        }
    }
}

pub(crate) fn name_to_table_name(name: &str) -> String {
    to_singular(&name.replace('"', "")).to_pascal_case()
}
