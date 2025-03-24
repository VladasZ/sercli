use std::{collections::HashMap, fs::read_to_string};

use anyhow::{Result, bail};
use inflector::Inflector;
use sqlparser::{
    ast::{CreateTable, Statement},
    dialect::PostgreSqlDialect,
    parser::Parser,
};
use utils::git_root;

use crate::entity::Entity;

const DIALECT: PostgreSqlDialect = PostgreSqlDialect {};

pub struct Migrations {
    pub model: HashMap<String, Entity>,
}

impl Migrations {}

impl Migrations {
    pub fn get() -> Result<Self> {
        let mut migrations = Self {
            model: HashMap::default(),
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
pub use {mod_name}::*;"
            ));
        }

        code
    }

    fn process_migration(&mut self, sql: &str) -> Result<()> {
        for statement in Parser::parse_sql(&DIALECT, sql)? {
            self.process_statement(statement)?;
        }

        Ok(())
    }

    fn process_statement(&mut self, statement: Statement) -> Result<()> {
        match statement {
            Statement::CreateTable(create) => self.process_create_table(create),
            _ => bail!("Unsupported statement: {statement}"),
        }
    }
}

impl Migrations {
    fn process_create_table(&mut self, create: CreateTable) -> Result<()> {
        let entity: Entity = create.into();

        if self.model.contains_key(&entity.name) {
            bail!("Duplicated entity name. '{}' already exists", entity.name)
        }

        self.model.insert(entity.name.clone(), entity);

        Ok(())
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
            [(
                "User".into(),
                Entity {
                    name:       "User".into(),
                    table_name: "users".into(),
                    fields:     vec![
                        Field {
                            name: "id".into(),
                            ty:   "i64",
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
                        }
                    ],
                }
            )]
            .into_iter()
            .collect()
        );

        println!("{}", migrations.model.get("User").unwrap().to_code());

        assert_eq!(
            migrations.model.get("User").unwrap().to_code(),
            r"
#[derive(Debug, PartialEq, Reflected)]
pub struct User {
   pub id: usize,
   pub email: String,
   pub age: i16,
   pub name: String,
   pub password: String,
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
            r"mod user
pub use user::*"
        );

        Ok(())
    }
}
