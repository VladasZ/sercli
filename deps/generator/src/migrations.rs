use std::{collections::HashMap, fs::read_to_string, path::PathBuf, process::Command};

use anyhow::{Result, bail};
use sqlparser::{
    ast::{CreateTable, Statement},
    dialect::PostgreSqlDialect,
    parser::Parser,
};

use crate::{entity::Entity, field::Field};

const DIALECT: PostgreSqlDialect = PostgreSqlDialect {};

pub struct Migrations {
    model: HashMap<String, Entity>,
}

impl Migrations {}

impl Migrations {
    pub fn new() -> Result<Self> {
        let mut migrations = Self {
            model: HashMap::default(),
        };

        for sql in get_sql()? {
            migrations.process_migration(&sql)?;
        }

        Ok(migrations)
    }

    fn process_migration(&mut self, sql: &str) -> Result<()> {
        for statement in Parser::parse_sql(&DIALECT, &sql)? {
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

fn git_root() -> Result<PathBuf> {
    let output = Command::new("git").args(["rev-parse", "--show-toplevel"]).output()?;

    if !output.status.success() {
        bail!("Failed to get Git repository root path");
    }

    assert!(output.status.success(), "Failed to get Git repository root path");
    let git_root = String::from_utf8_lossy(&output.stdout).trim_end_matches('\n').to_string();

    Ok(PathBuf::from(git_root))
}

#[test]
fn test() -> Result<()> {
    let migrations = Migrations::new()?;

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
                        ty:   "usize",
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

    Ok(())
}
