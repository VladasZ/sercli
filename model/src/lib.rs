mod entities;
mod requests;
mod test_data;
mod user;

pub use entities::*;
pub use requests::*;

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use sercli::db::prepare_db;

    use crate::Model;

    #[ignore]
    #[tokio::test]
    async fn setup_db() -> Result<()> {
        use sercli::db::{generate_model, prepare_db};

        generate_model("../model/migrations")?;
        prepare_db("../model/migrations").await?;

        Ok(())
    }

    #[ignore]
    #[tokio::test]
    async fn wipe_db() -> Result<()> {
        let pool = prepare_db("../model/migrations").await?;
        Model::drop_all_tables(&pool).await?;
        sercli::db::stop_containers()
    }
}
