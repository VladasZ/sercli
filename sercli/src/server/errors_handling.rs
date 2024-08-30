use anyhow::anyhow;
use axum::Json;
use reflected::Reflected;

use crate::server::AppError;

/// Handle database errors and return user readable response
pub trait ToResponse<T: Reflected> {
    fn to_response(self) -> Result<Json<T>, AppError>;
}

impl<T: Reflected> ToResponse<T> for Result<T, sqlx::Error> {
    fn to_response(self) -> Result<Json<T>, AppError> {
        match self {
            Ok(object) => Ok(Json(object)),
            Err(error) => {
                let error = format!("{error}");
                Err(anyhow!(parse_error::<T>(error)).into())
            }
        }
    }
}

fn parse_error<T: Reflected>(err: String) -> String {
    if err.contains("duplicate key value violates unique constraint") {
        parse_unique_violation::<T>(err)
    } else {
        err
    }
}

fn parse_unique_violation<T: Reflected>(err: String) -> String {
    let field_description = extract_substring_in(&err, '"').expect("Unique field description not found");
    let field_name = extract_substring_in(&field_description, '_').expect("Unique field name not found");

    format!("{} with such {field_name} already exists.", T::type_name())
}

fn extract_substring_in(input: &str, symbol: char) -> Option<String> {
    if let Some(start) = input.find(symbol) {
        if let Some(end) = input[start + 1..].find(symbol) {
            let extracted = &input[start + 1..start + 1 + end];
            return Some(extracted.to_string());
        }
    }
    None
}
