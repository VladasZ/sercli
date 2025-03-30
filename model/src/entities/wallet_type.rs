
#[derive(strum::Display, strum::EnumString, serde::Serialize, serde::Deserialize, sqlx::Type, Copy, Clone, Default, PartialEq, Debug)]
#[sqlx(type_name = "wallet_type", rename_all = "lowercase")]
pub enum WalletType {
    #[default]
    Fiat,
    Crypto,
}

impl sercli::reflected::ToReflectedVal<WalletType> for &str {
    fn to_reflected_val(&self) -> Result<WalletType, String> {
        use std::str::FromStr;
        Ok(WalletType::from_str(self).unwrap())
    }
}
