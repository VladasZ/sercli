use std::{
    fmt::Write,
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::Result;
use inflector::Inflector;
use sqlparser::ast::{ObjectName, UserDefinedTypeRepresentation};

use crate::entity::name_to_table_name;

pub struct PgEnum {
    pub name:       String,
    pub table_name: String,
    pub cases:      Vec<String>,
}

impl PgEnum {
    pub fn generate_file(&self, folder: &Path) -> anyhow::Result<()> {
        use std::io::Write;

        let path = folder.join(self.file_name());

        let mut file = File::create_new(&path)?;

        file.write_all(self.to_code()?.as_bytes())?;

        Ok(())
    }

    pub(crate) fn to_code(&self) -> Result<String> {
        let name = &self.name;
        let table_name = &self.table_name;

        let mut fields = String::new();

        for field in &self.cases {
            writeln!(fields, "    {field},")?;
        }

        Ok(format!(
            r#"
#[derive(strum::Display, strum::EnumString, serde::Serialize, serde::Deserialize, sqlx::Type, Copy, Clone, Default, PartialEq, Debug)]
#[sqlx(type_name = {table_name}, rename_all = "lowercase")]
pub enum {name} {{
    #[default]
{fields}}}

impl sercli::reflected::ToReflectedVal<{name}> for &str {{
    fn to_reflected_val(&self) -> Result<{name}, String> {{
        use std::str::FromStr;
        Ok({name}::from_str(self).unwrap())
    }}
}}
"#
        ))
    }

    fn file_name(&self) -> PathBuf {
        format!("{}.rs", self.name.to_snake_case()).into()
    }

    fn parse_cases(repr: UserDefinedTypeRepresentation) -> Vec<String> {
        let UserDefinedTypeRepresentation::Enum { labels } = repr else {
            panic!("Unsupported enum representation: {repr}")
        };

        labels.into_iter().map(|l| l.value.to_pascal_case()).collect()
    }
}

impl From<(ObjectName, UserDefinedTypeRepresentation)> for PgEnum {
    fn from(value: (ObjectName, UserDefinedTypeRepresentation)) -> Self {
        let table_name = format!("{}", value.0);

        Self {
            name: name_to_table_name(&table_name),
            table_name,
            cases: Self::parse_cases(value.1),
        }
    }
}
