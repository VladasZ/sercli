use std::{collections::BTreeMap, fs::read_to_string};

use anyhow::Result;
use inflector::Inflector;
use sercli_utils::git_root;
use sqlparser::{
    ast::{AlterTableOperation, CreateTable, HiveSetLocation, Ident, ObjectName, Statement},
    dialect::PostgreSqlDialect,
    parser::Parser,
};

use crate::entity::Entity;

const DIALECT: PostgreSqlDialect = PostgreSqlDialect {};

pub struct Migrations {
    pub model: BTreeMap<String, Entity>,
}

impl Migrations {}

impl Migrations {
    pub fn get() -> Result<Self> {
        let mut migrations = Self {
            model: BTreeMap::default(),
        };

        for sql in get_sql()? {
            migrations.process_migration(&sql)?;
        }

        Ok(migrations)
    }

    pub fn mod_code(&self) -> String {
        let mut code = String::new();

        for entity in self.model.values() {
            let mod_name = entity.name.to_snake_case();

            code.push_str(&format!(
                r"mod {mod_name};
pub use {mod_name}::*;
"
            ));
        }

        code
    }

    fn process_migration(&mut self, sql: &str) -> Result<()> {
        for statement in Parser::parse_sql(&DIALECT, sql)? {
            self.process_statement(statement);
        }

        Ok(())
    }

    fn process_statement(&mut self, statement: Statement) {
        match statement {
            Statement::CreateTable(create) => self.process_create_table(create),
            Statement::AlterTable {
                name,
                if_exists,
                only,
                operations,
                location,
                on_cluster,
            } => self.process_alter_table(name, if_exists, only, operations, location, on_cluster),
            _ => unimplemented!("Unsupported statement: {statement}"),
        }
    }
}

impl Migrations {
    fn process_create_table(&mut self, create: CreateTable) {
        let entity: Entity = create.into();

        if self.model.contains_key(&entity.name) {
            panic!("Duplicated entity name. '{}' already exists", entity.name)
        }

        self.model.insert(entity.name.clone(), entity);
    }

    fn process_alter_table(
        &mut self,
        name: ObjectName,
        _if_exists: bool,
        _only: bool,
        operations: Vec<AlterTableOperation>,
        _location: Option<HiveSetLocation>,
        _on_cluster: Option<Ident>,
    ) {
        let entity: Entity = name.into();

        let Some(existing_entity) = self.model.get_mut(&entity.name) else {
            panic!("Entity: {entity:?} doesn't exist yet to alter it")
        };

        existing_entity.process_alter_table_operations(operations);
    }
}

fn get_sql() -> Result<impl Iterator<Item = String>> {
    let path = git_root()?.join("model/migrations");

    let mut result = vec![];

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();

        if file_path.is_file() {
            result.push(read_to_string(&file_path)?);
        }
    }

    Ok(result.into_iter())
}

#[cfg(test)]
mod test {
    use crate::{entity::Entity, field::Field, migrations::Migrations};

    #[test]
    fn entities() -> anyhow::Result<()> {
        let migrations = Migrations::get()?;

        assert_eq!(
            migrations.model,
            [
                (
                    "User".into(),
                    Entity {
                        name:       "User".into(),
                        table_name: "users".into(),
                        fields:     vec![
                            Field {
                                name: "id".into(),
                                ty:   "sercli::ID",
                            },
                            Field {
                                name: "email".into(),
                                ty:   "String",
                            },
                            Field {
                                name: "age".into(),
                                ty:   "i16",
                            },
                            Field {
                                name: "name".into(),
                                ty:   "String",
                            },
                            Field {
                                name: "password".into(),
                                ty:   "String",
                            },
                            Field {
                                name: "birthday".into(),
                                ty:   "sercli::DateTime",
                            }
                        ],
                    }
                ),
                (
                    "Wallet".into(),
                    Entity {
                        name:       "Wallet".into(),
                        table_name: "wallets".into(),
                        fields:     vec![
                            Field {
                                name: "id".into(),
                                ty:   "sercli::ID",
                            },
                            Field {
                                name: "user_id".into(),
                                ty:   "i32",
                            },
                            Field {
                                name: "name".into(),
                                ty:   "String",
                            },
                            Field {
                                name: "amount".into(),
                                ty:   "sercli::Decimal",
                            },
                        ],
                    }
                )
            ]
            .into_iter()
            .collect()
        );

        println!("{}", migrations.model.get("User").unwrap().to_code());

        assert_eq!(
            migrations.model.get("User").unwrap().to_code(),
            r"
mod reflected {
    pub use sercli::reflected::*;
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize, reflected::Reflected, sqlx::FromRow)]
pub struct User {
   pub id: sercli::ID,
   pub email: String,
   pub age: i16,
   pub name: String,
   pub password: String,
   pub birthday: sercli::DateTime,
}
"
        );

        Ok(())
    }

    #[test]
    fn mod_code() -> anyhow::Result<()> {
        let migrations = Migrations::get()?;

        println!("{}", migrations.mod_code());

        assert_eq!(
            migrations.mod_code(),
            r"mod user;
pub use user::*;
mod wallet;
pub use wallet::*;
"
        );

        Ok(())
    }
}
