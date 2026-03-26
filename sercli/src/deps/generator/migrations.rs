use std::{
    collections::BTreeMap,
    fmt::Write,
    fs::{DirEntry, read_to_string},
    path::Path,
};

use anyhow::{Context, Result};
use inflector::Inflector;
use sqlparser::{
    ast::{
        AlterTableOperation, CreateTable, HiveSetLocation, Ident, ObjectName, Statement,
        UserDefinedTypeRepresentation,
    },
    dialect::PostgreSqlDialect,
    parser::Parser,
};

use crate::deps::generator::{entity::Entity, pg_enum::PgEnum};

const DIALECT: PostgreSqlDialect = PostgreSqlDialect {};

#[derive(Debug)]
pub struct Migrations {
    pub entities: BTreeMap<String, Entity>,
    pub enums:    BTreeMap<String, PgEnum>,
}

impl Migrations {}

impl Migrations {
    pub fn get(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let expanded;
        let path = if let Ok(rest) = path.strip_prefix("~") {
            let home = std::env::var("HOME").context("No HOME")?;
            expanded = Path::new(&home).join(rest);
            expanded.as_path()
        } else {
            path
        };

        let mut migrations = Self {
            entities: BTreeMap::default(),
            enums:    BTreeMap::default(),
        };

        for sql in get_sql(path)? {
            migrations.process_migration(&sql)?;
        }

        Ok(migrations)
    }

    pub fn mod_code(&self) -> Result<String> {
        let mut code = "mod model;\n#[allow(unused_imports)]\npub use model::*;\n".to_string();

        for en in self.enums.values() {
            let mod_name = en.name.to_snake_case();

            write!(
                code,
                r"mod {mod_name};
pub use {mod_name}::*;
"
            )?;
        }

        for entity in self.entities.values() {
            let mod_name = entity.name.to_snake_case();

            write!(
                code,
                r"mod {mod_name};
pub use {mod_name}::*;
"
            )?;
        }

        Ok(code)
    }

    pub fn model_code(&self) -> Result<String> {
        let mut tables = String::new();

        for entity in self.entities.values() {
            writeln!(tables, "            \"{}\",", entity.table_name)?;
        }

        tables.pop();

        Ok(format!(
            r#"
#![allow(dead_code)]

use anyhow::Result;

pub struct Model;

impl Model {{
    pub fn tables() -> &'static [&'static str] {{
        &[
{tables}
        ]
    }}

    pub async fn drop_all_tables(pool: &sqlx::PgPool) -> Result<()> {{
        use sqlx::Executor;

        for table in Self::tables() {{
            pool.execute(&*format!("DROP TABLE IF EXISTS {{table}} CASCADE;")).await?;
        }}

        Ok(())
    }}
}}
        "#
        ))
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
            Statement::AlterTable(alter) => self.process_alter_table(
                alter.name,
                alter.if_exists,
                alter.only,
                alter.operations,
                alter.location,
                alter.on_cluster,
            ),
            Statement::CreateType { name, representation } => self.process_create_type(name, representation),
            Statement::CreateIndex(_) => (),
            _ => unimplemented!("Unsupported statement: {statement:?}"),
        }
    }
}

impl Migrations {
    fn process_create_table(&mut self, create: CreateTable) {
        let entity: Entity = create.into();

        if self.entities.contains_key(&entity.name) {
            panic!("Duplicated entity name. '{}' already exists", entity.name)
        }

        self.entities.insert(entity.name.clone(), entity);
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

        let Some(existing_entity) = self.entities.get_mut(&entity.name) else {
            panic!("Entity: {entity:?} doesn't exist yet to alter it")
        };

        existing_entity.process_alter_table_operations(operations);
    }

    fn process_create_type(
        &mut self,
        name: ObjectName,
        representation: Option<UserDefinedTypeRepresentation>,
    ) {
        let en: PgEnum = (name, representation).into();

        self.enums.insert(en.name.clone(), en);
    }
}

fn get_sql(path: impl AsRef<Path>) -> Result<impl Iterator<Item = String>> {
    let path = path.as_ref();

    let mut entries: Vec<_> = std::fs::read_dir(path)
        .with_context(|| format!("Trying to read: {}", path.display()))?
        .filter_map(Result::ok)
        .collect();

    entries.sort_by_key(DirEntry::path);

    let mut result = vec![];

    for entry in entries {
        let file_path = entry.path();

        if file_path.is_file() {
            result.push(read_to_string(&file_path)?);
        }
    }

    Ok(result.into_iter())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::deps::generator::relation::Relation;

    #[test]
    fn parses_relations() -> anyhow::Result<()> {
        let mut migrations = Migrations {
            entities: BTreeMap::default(),
            enums:    BTreeMap::default(),
        };

        migrations.process_migration(
            r#"
            CREATE TABLE "users" ("telegram_id" bigint NOT NULL);
            CREATE TABLE "posts" ("id" bigint NOT NULL, "user_id" bigint NOT NULL);
            ALTER TABLE "posts" ADD FOREIGN KEY ("user_id") REFERENCES "users" ("telegram_id");
            "#,
        )?;

        let post = migrations.entities.get("Post").expect("Post entity not found");
        assert_eq!(
            post.relations,
            vec![Relation {
                field:      "user_id".into(),
                references: "User".into(),
            }]
        );

        let user = migrations.entities.get("User").expect("User entity not found");
        assert!(user.relations.is_empty());

        Ok(())
    }

    #[test]
    fn entities() -> anyhow::Result<()> {
        //         let migrations = Migrations::get()?;
        //
        //         // dbg!(&migrations.entities);
        //
        //         assert_eq!(
        //             migrations.entities,
        //             [
        //                 (
        //                     "User".into(),
        //                     Entity {
        //                         name: "User".into(),
        //                         table_name: "users".into(),
        //                         fields: vec![
        //                             Field {
        //                                 name: "id".into(),
        //                                 ty: "sercli::ID".into(),
        //                             },
        //                             Field {
        //                                 name: "email".into(),
        //                                 ty: "String".into(),
        //                             },
        //                             Field {
        //                                 name: "password".into(),
        //                                 ty: "String".into(),
        //                             },
        //                             Field {
        //                                 name: "age".into(),
        //                                 ty: "i32".into(),
        //                             },
        //                             Field {
        //                                 name: "birthday".into(),
        //                                 ty: "Option<DateTime>".into(),
        //                             }
        //                         ],
        //                     }
        //                 ),
        //                 (
        //                     "Wallet".into(),
        //                     Entity {
        //                         name: "Wallet".into(),
        //                         table_name: "wallets".into(),
        //                         fields: vec![
        //                             Field {
        //                                 name: "id".into(),
        //                                 ty: "sercli::ID".into(),
        //                             },
        //                             Field {
        //                                 name: "user_id".into(),
        //                                 ty: "i32".into(),
        //                             },
        //                             Field {
        //                                 name: "name".into(),
        //                                 ty: "String".into(),
        //                             },
        //                             Field {
        //                                 name: "amount".into(),
        //                                 ty: "sercli::Decimal".into(),
        //                             },
        //                             Field {
        //                                 name: "tp".into(),
        //                                 ty: "crate::WalletType".into(),
        //                             },
        //                         ],
        //                     }
        //                 )
        //             ]
        //             .into_iter()
        //             .collect()
        //         );
        //
        //         println!("{}", migrations.entities.get("User").unwrap().to_code());
        //
        //         assert_eq!(
        //             migrations.entities.get("User").unwrap().to_code(),
        //             r"
        // mod reflected {
        //     pub use sercli::reflected::*;
        // }
        //
        // #[derive(Debug, Default, Clone, PartialEq, serde::Serialize,
        // serde::Deserialize, reflected::Reflected, sqlx::FromRow)] pub struct
        // User {     pub id: sercli::ID,
        //     pub email: String,
        //     pub password: String,
        //     pub age: i32,
        //     pub birthday: sercli::DateTime,
        // }
        // "
        //         );
        //
        //         Ok(())
        //     }
        //
        //     #[test]
        //     fn mod_code() -> anyhow::Result<()> {
        //         let migrations = Migrations::get()?;
        //
        //         println!("{}", migrations.mod_code());
        //
        //         assert_eq!(
        //             migrations.mod_code(),
        //             r"mod wallet_type;
        // pub use wallet_type::*;
        // mod user;
        // pub use user::*;
        // mod wallet;
        // pub use wallet::*;
        // "
        //         );

        Ok(())
    }
}
