use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use inflector::{Inflector, string::singularize::to_singular};
use sqlparser::ast::{AlterTableOperation, CreateTable, ObjectName, ObjectNamePart, TableConstraint};

use crate::deps::generator::{
    field::Field,
    relation::{InverseRelation, Relation},
};

#[derive(Debug, PartialEq)]
pub struct Entity {
    pub name:              String,
    pub table_name:        String,
    pub fields:            Vec<Field>,
    pub relations:         Vec<Relation>,
    pub inverse_relations: Vec<InverseRelation>,
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
            AlterTableOperation::AddConstraint {
                constraint,
                not_valid: _,
            } => {
                if let TableConstraint::ForeignKey(fk) = constraint {
                    let foreign_table = format!("{}", fk.foreign_table).replace('"', "");
                    for (col, ref_col) in fk.columns.into_iter().zip(fk.referred_columns.into_iter()) {
                        self.relations.push(Relation {
                            field:            col.value,
                            references:       name_to_table_name(&foreign_table),
                            references_field: ref_col.value,
                        });
                    }
                }
            }
            _ => unimplemented!("Unsipported alter table operation: {operation}"),
        }
    }

    pub(crate) fn to_code(&self) -> String {
        let name = &self.name;

        let mut fields = String::new();

        for field in &self.fields {
            fields.push_str(&field.to_code());
        }

        let mut relation_imports = String::new();
        let mut relation_methods = String::new();

        for inv in &self.inverse_relations {
            let method_name = &inv.table_name;
            let entity_name = &inv.entity_name;
            let fk_field = &inv.fk_field;
            let local_field = &inv.local_field;

            relation_imports.push_str(&format!("use crate::{entity_name};\n"));
            relation_methods.push_str(&format!(
                r#"
    pub async fn {method_name}(&self, pool: &sqlx::PgPool) -> anyhow::Result<Vec<{entity_name}>> {{
        Ok(sqlx::query_as("SELECT * FROM {method_name} WHERE {fk_field} = $1")
            .bind(self.{local_field})
            .fetch_all(pool)
            .await?)
    }}
"#
            ));
        }

        let impl_block = if relation_methods.is_empty() {
            String::new()
        } else {
            format!("\nimpl {name} {{{relation_methods}}}\n")
        };

        format!(
            r"
#[allow(unused_imports)]
#[allow(clippy::wildcard_imports)]
use sercli::*;
{relation_imports}
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
{fields}}}{impl_block}"
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
            relations: vec![],
            inverse_relations: vec![],
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
            ObjectNamePart::Function(_) => unimplemented!(),
        };

        Self {
            name: name_to_table_name(&table_name),
            table_name,
            fields: vec![],
            relations: vec![],
            inverse_relations: vec![],
        }
    }
}

pub(crate) fn name_to_table_name(name: &str) -> String {
    to_singular(&name.replace('"', "")).to_pascal_case()
}
