use anyhow::Result;
use sercli::db::{generate_model, prepare_db};

#[tokio::main]
pub async fn main() -> Result<()> {
    unsafe { std::env::set_var("REBUILD", format!("{:?}", std::time::Instant::now())) };
    println!("cargo:rerun-if-env-changed=REBUILD");
    println!("cargo:rerun-if-changed=build.rs");
    generate_model()?;
    prepare_db().await?;
    Ok(())
}
