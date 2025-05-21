use inflector::Inflector;
use reflected::{Field, Reflected, Type};
use sqlx::{FromRow, postgres::PgRow};

pub trait Entity: Sized + Reflected + for<'r> FromRow<'r, PgRow> + Unpin {
    fn table_name() -> String;
    fn create_table_query() -> String;
    fn insert_query() -> String;
}

impl<T: Reflected + for<'r> FromRow<'r, PgRow> + Unpin> Entity for T {
    fn table_name() -> String {
        Self::type_name().to_plural().to_snake_case()
    }

    fn create_table_query() -> String {
        let table_name = Self::table_name();

        if T::fields().is_empty() {
            return format!(
                r"CREATE TABLE IF NOT EXISTS {table_name}
(
   id SERIAL PRIMARY KEY
);"
            );
        }

        let mut fields = String::new();

        for field in T::fields() {
            if field.is_id() {
                continue;
            }
            fields.push_str(&field_to_sql(field));
        }

        fields.pop();
        fields.pop();

        format!(
            r"CREATE TABLE IF NOT EXISTS {table_name}
(
   id SERIAL PRIMARY KEY,
{fields}
);"
        )
    }

    fn insert_query() -> String {
        let fields: Vec<_> = T::fields().iter().filter(|f| !f.is_id()).collect();

        let columns: Vec<_> = fields.iter().map(|field| field.name.to_string()).collect();
        let columns = columns.join(", ");

        let placeholders = (1..=fields.len()).map(|i| format!("${i}")).collect::<Vec<String>>().join(", ");

        format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *;",
            T::table_name(),
            columns,
            placeholders
        )
    }
}

fn field_to_sql<T>(field: &'static Field<T>) -> String {
    format!("   {} {},\n", field.name, sql_type_from_field(field))
}

fn sql_type_from_field<T>(field: &'static Field<T>) -> String {
    match field.tp {
        Type::Float => "REAL NOT NULL".into(),
        Type::Integer => "INTEGER NOT NULL".into(),
        Type::Text => "VARCHAR(255) NOT NULL".into(),
        Type::Enum => {
            format!("{} NOT NULL", field.type_name.to_snake_case())
        }
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod test {
    use reflected::{Reflected, ToReflectedVal};
    use sqlx::FromRow;

    use crate::Entity;

    #[derive(
        strum::Display,
        strum::EnumString,
        serde::Serialize,
        serde::Deserialize,
        sqlx::Type,
        Copy,
        Clone,
        Default,
        PartialEq,
        Debug,
    )]
    #[sqlx(type_name = "wallet_type", rename_all = "lowercase")]
    pub enum WalletType {
        #[default]
        Fiat,
        Crypto,
    }

    impl ToReflectedVal<WalletType> for &str {
        fn to_reflected_val(&self) -> std::result::Result<WalletType, String> {
            use std::str::FromStr;
            Ok(WalletType::from_str(self).unwrap())
        }
    }

    #[derive(Default, Reflected, FromRow)]
    struct Cat {
        age:    i32,
        name:   String,
        weight: f32,
        tp:     WalletType,
    }

    #[test]
    fn table_name() {
        #[derive(Default, Reflected, FromRow)]
        struct Cat {}
        assert_eq!(Cat::table_name(), "cats");

        #[derive(Default, Reflected, FromRow)]
        struct BigCat {}
        assert_eq!(BigCat::table_name(), "big_cats");

        #[derive(Default, Reflected, FromRow)]
        struct PremiumPackage {
            a:    i32,
            name: String,
            sss:  f32,
            tp:   WalletType,
        }
        assert_eq!(PremiumPackage::table_name(), "premium_packages");

        dbg!(PremiumPackage::create_table_query());
    }

    #[test]
    fn create_table_query() {
        #[derive(Default, Reflected, FromRow)]
        struct Empty {}

        println!("{}", Empty::create_table_query());

        assert_eq!(
            Empty::create_table_query(),
            r"CREATE TABLE IF NOT EXISTS empties
(
   id SERIAL PRIMARY KEY
);"
        );

        println!("{}", Cat::create_table_query());

        assert_eq!(
            Cat::create_table_query(),
            r"CREATE TABLE IF NOT EXISTS cats
(
   id SERIAL PRIMARY KEY,
   age INTEGER NOT NULL,
   name VARCHAR(255) NOT NULL,
   weight REAL NOT NULL,
   tp wallet_type NOT NULL
);"
        );
    }

    #[test]
    fn insert_query() {
        println!("{}", Cat::insert_query());
    }
}
