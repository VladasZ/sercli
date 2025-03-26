use anyhow::{Result, anyhow};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tokio::task::spawn_blocking;

pub async fn hash_password(pass: &str) -> Result<String> {
    let pass = pass.to_owned();

    spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);

        let password_hash = Argon2::default()
            .hash_password(pass.as_bytes(), &salt)
            .map_err(|e| anyhow!(e))?
            .to_string();

        Ok(password_hash)
    })
    .await?
}

pub async fn check_password(pass: &str, hash: &str) -> Result<()> {
    let pass = pass.to_owned();
    let hash = hash.to_owned();

    spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&hash).map_err(|e| anyhow!(e))?;
        Argon2::default()
            .verify_password(pass.as_bytes(), &parsed_hash)
            .map_err(|e| anyhow!(e))?;
        Ok(())
    })
    .await?
}

#[cfg(test)]
mod test {

    use std::time::Instant;

    use anyhow::Result;
    use fake::{Fake, Faker};

    use crate::password::{check_password, hash_password};

    #[tokio::test]
    async fn hash_and_check_password() -> Result<()> {
        for _ in 0..5 {
            let pass = Faker.fake::<String>();

            let hash = hash_password(&pass).await?;

            check_password(&pass, &hash).await?;

            dbg!(&pass);
        }

        Ok(())
    }

    fn _log_execution_time<F, T>(func: F) -> T
    where F: FnOnce() -> T {
        let start = Instant::now();
        let result = func();
        let duration = start.elapsed();
        println!("Execution time: {:?} ms", duration.as_millis());
        result
    }
}
