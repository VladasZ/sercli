use anyhow::{Result, anyhow, bail};
use fake::Fake;
use log::error;
use pasetors::{
    Local,
    claims::{Claims, ClaimsValidationRules},
    keys::SymmetricKey,
    local,
    token::UntrustedToken,
    version4::V4,
};
use sqlx::{Executor, FromRow, PgPool, query, query_as};

use crate::{DBStorage, ID, SercliUser, server::crud::Crud};

#[derive(Debug, FromRow)]
struct UserToken {
    user_id: ID,
    token:   String,
}

pub(crate) struct AccessToken {}

impl AccessToken {
    pub async fn generate_token<User: SercliUser>(user: &User, pool: &PgPool) -> Result<String> {
        Self::create_table(pool).await?;

        let key = Self::get_encryption_key(pool).await?;
        let token = Self::create_token(user, pool).await?;

        let mut claims = Claims::new()?;
        claims.non_expiring();
        claims.add_additional("user_id", user.id())?;
        claims.add_additional("user_login", user.login())?;
        claims.add_additional("user_token", token)?;

        Ok(local::encrypt(&key, &claims, None, None)?)
    }

    pub async fn check_token<User: SercliUser>(token: &str, pool: &PgPool) -> Result<User> {
        Self::create_table(pool).await?;

        let key = Self::get_encryption_key(pool).await?;

        let mut validation_rules = ClaimsValidationRules::new();
        validation_rules.allow_non_expiring();

        let untrusted_token = UntrustedToken::<Local, V4>::try_from(&token.to_string())?;
        let trusted_token = local::decrypt(&key, &untrusted_token, &validation_rules, None, None)?;
        let claims = trusted_token.payload_claims().ok_or_else(|| anyhow!("No claims"))?;

        let user_id: ID = claims
            .get_claim("user_id")
            .ok_or_else(|| anyhow!("No user_id in claim"))?
            .as_i64()
            .ok_or_else(|| anyhow!("Invalid value in user_id"))?
            .try_into()?;

        let user_login: &str = claims
            .get_claim("user_login")
            .ok_or_else(|| anyhow!("No user_login in claim"))?
            .as_str()
            .ok_or_else(|| anyhow!("Invalid value in user_login"))?;

        let user_token: &str = claims
            .get_claim("user_token")
            .ok_or_else(|| anyhow!("No user_token in claim"))?
            .as_str()
            .ok_or_else(|| anyhow!("Invalid value in user_token"))?;

        let user = User::with_id(user_id, pool).await?;

        if user_login != user.login() {
            bail!("Invalid user login in claim")
        }

        let token: Option<UserToken> = query_as("SELECT * FROM token_storage WHERE token = $1")
            .bind(user_token)
            .fetch_optional(pool)
            .await?;

        let Some(token) = token else {
            bail!("This token is not valid anymore")
        };

        // This is a security incident. It means that encryption key has leaked.
        // Think of a way to notify about this
        if token.user_id != user.id() {
            error!("This is bad");
            bail!("Invalid user id in token");
        }

        Ok(user)
    }

    #[allow(dead_code)]
    pub async fn invalidate_all_tokens<User: SercliUser>(user: &User, pool: &PgPool) -> Result<()> {
        pool.execute(query("DELETE FROM token_storage WHERE user_id = $1").bind(user.id()))
            .await?;

        Ok(())
    }

    async fn get_encryption_key(pool: &PgPool) -> Result<SymmetricKey<V4>> {
        use pasetors::keys::Generate;

        const STORAGE_KEY: &str = "access_token_encryption_key";

        if let Some(data) = DBStorage::get(STORAGE_KEY, pool).await? {
            return Ok(SymmetricKey::<V4>::from(&data)?);
        }

        let key = SymmetricKey::<V4>::generate()?;

        DBStorage::set(STORAGE_KEY, key.as_bytes(), pool).await?;

        Ok(key)
    }

    async fn create_token<User: SercliUser>(user: &User, pool: &PgPool) -> Result<String> {
        let token = UserToken {
            user_id: user.id(),
            token:   32.fake(),
        };

        pool.execute(
            query("INSERT INTO token_storage (user_id, token) VALUES($1, $2)")
                .bind(token.user_id)
                .bind(&token.token),
        )
        .await?;

        Ok(token.token)
    }

    async fn create_table(pool: &PgPool) -> Result<()> {
        pool.execute(query(
            r"CREATE TABLE IF NOT EXISTS token_storage (
                   id SERIAL       PRIMARY KEY,
              user_id INTEGER      NOT NULL,
                token VARCHAR(255) NOT NULL
);",
        ))
        .await
        .map_err(|e| anyhow!(e))?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use fake::{Fake, faker::internet::en::SafeEmail};
    use reflected::Reflected;
    use sqlx::FromRow;

    use crate::{Crud, ID, SercliUser, db::prepare_db, server::access_token::AccessToken};

    #[derive(Debug, Default, Clone, Reflected, FromRow)]
    struct SomeUser {
        id:    ID,
        email: String,
    }

    impl SercliUser for SomeUser {
        fn id(&self) -> ID {
            self.id
        }

        fn password(&self) -> &str {
            todo!()
        }

        fn login(&self) -> &str {
            &self.email
        }

        fn login_field_name() -> &'static str {
            todo!()
        }
    }

    #[tokio::test]
    async fn generate_token() -> Result<()> {
        let pool = prepare_db().await?;

        let user = SomeUser {
            id:    0,
            email: SafeEmail().fake(),
        };

        SomeUser::create_table(&pool).await?;

        let user = user.insert(&pool).await?;

        AccessToken::invalidate_all_tokens(&user, &pool).await?;

        let token = AccessToken::generate_token(&user, &pool).await?;

        let authorized_user: SomeUser = AccessToken::check_token(&token, &pool).await?;

        assert_eq!(user.email, authorized_user.email);

        AccessToken::invalidate_all_tokens(&user, &pool).await?;

        let error = AccessToken::check_token::<SomeUser>(&token, &pool)
            .await
            .expect_err("No error on invalidated token");

        assert!(format!("{error}").contains("This token is not valid anymore"));

        Ok(())
    }
}
