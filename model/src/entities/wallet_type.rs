
#[derive(strum::Display, strum::EnumString, serde::Serialize, serde::Deserialize, sqlx::Type, Copy, Clone, Default, PartialEq, Debug)]
#[sqlx(type_name = "wallet_type", rename_all = "lowercase")]
pub enum WalletType {
    #[default]
    Fiat,
    Crypto,
}
