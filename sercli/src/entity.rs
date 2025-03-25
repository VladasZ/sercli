use inflector::Inflector;
use reflected::{Field, Reflected, Type};
use sqlx::{FromRow, postgres::PgRow};

pub trait Entity: Sized + Reflected + for<'r> FromRow<'r, PgRow> + Unpin {
    fn table_name() -> String;
    fn create_table_query() -> String;
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
            fields.push_str(&field_to_sql(field));
        }

        fields.pop();
        fields.pop();

        format!(
            r"CREATE TABLE IF NOT EXISTS {table_name}
(
   id SERIAL PRIMARY KEY,
{fields}
);
"
        )
    }
}

fn field_to_sql<T>(field: &'static Field<T>) -> String {
    format!("   {} {},\n", field.name, sql_type_from_field(field.tp))
}

fn sql_type_from_field(tp: Type) -> &'static str {
    match tp {
        Type::Float => "REAL NOT NULL",
        Type::Integer => "INTEGER NOT NULL",
        Type::Text => "VARCHAR(255) NOT NULL",
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod test {
    use reflected::Reflected;
    use sqlx::FromRow;

    use crate::Entity;

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
        }
        assert_eq!(PremiumPackage::table_name(), "premium_packages");

        dbg!(PremiumPackage::create_table_query());
    }

    #[test]
    fn create_table_query() {
        #[derive(Default, Reflected, FromRow)]
        struct Empty {}

        println!("{}", Empty::create_table_query());

        #[derive(Default, Reflected, FromRow)]
        struct Cat {
            age:    i32,
            name:   String,
            weight: f32,
        }

        println!("{}", Cat::create_table_query());
    }
}
