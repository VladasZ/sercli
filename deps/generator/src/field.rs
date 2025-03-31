use inflector::Inflector;
use sqlparser::ast::{ColumnDef, ColumnOption, DataType};

#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty:   String,
}

impl Field {
    pub fn to_code(&self) -> String {
        format!("    pub {}: {},\n", self.name, self.ty)
    }
}

impl From<ColumnDef> for Field {
    fn from(value: ColumnDef) -> Self {
        let non_null = value
            .options
            .iter()
            .any(|option| matches!(option.option, ColumnOption::NotNull));

        let ty = get_type(
            &value.data_type,
            non_null || value.name.to_string().replace('"', "") == "id",
        );

        Self {
            name: value.name.value,
            ty,
        }
    }
}

fn get_type(ty: &DataType, non_null: bool) -> String {
    let tp: String = match ty {
        DataType::Custom(object_name, _) => {
            let name = object_name.0.first().unwrap_or_else(|| {
                panic!("Empty object name: {object_name}");
            });

            let Some(ident) = name.as_ident() else {
                panic!("Failed to convert object name to ident: {object_name}");
            };

            if ident.value.to_lowercase() == "SERIAL".to_lowercase() {
                "ID".into()
            } else {
                format!("crate::{}", ident.value.to_pascal_case())
            }
        }
        DataType::Varchar(_) => "String".into(),
        DataType::SmallInt(_) => "i16".into(),
        DataType::Integer(_) => "i32".into(),
        DataType::Decimal(_) => "Decimal".into(),
        DataType::Timestamp(_, _) => "DateTime".into(),
        DataType::Real => "f32".into(),
        DataType::Interval => "Duration".into(),
        _ => panic!("Unsupported date type: {ty:?}"),
    };

    if non_null { tp } else { format!("Option<{tp}>") }
}
