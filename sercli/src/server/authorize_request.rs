use std::marker::PhantomData;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use fake::Fake;
use sqlx::PgPool;

use crate::{
    SercliUser,
    server::{AppError, TOKEN_STORAGE},
};

pub struct AuthorizeRequest<User: SercliUser> {
    _p: PhantomData<User>,
}

impl<User: SercliUser> AuthorizeRequest<User> {
    pub async fn generate_token(&self, user: &User) -> String {
        let token: String = 64.fake();

        TOKEN_STORAGE.lock().await.insert(token.clone(), user.id());

        token
    }
}

impl<S: Sync, User: SercliUser> FromRequestParts<S> for AuthorizeRequest<User>
where
    PgPool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self { _p: PhantomData })
    }
}

#[cfg(test)]
mod test {

    use anyhow::Result;

    #[test]
    fn generate_token() -> Result<()> {
        use core::convert::TryFrom;

        use pasetors::{
            Local,
            claims::{Claims, ClaimsValidationRules},
            keys::{Generate, SymmetricKey},
            local,
            token::UntrustedToken,
            version4::V4,
        };

        // Setup the default claims, which include `iat` and `nbf` as the current time
        // and `exp` of one hour. Add a custom `data` claim as well.
        let mut claims = Claims::new()?;
        claims.add_additional("data", "A secret, encrypted message")?;
        // claims.non_expiring();

        dbg!(&claims);

        // Generate the key and encrypt the claims.
        let sk = SymmetricKey::<V4>::generate()?;

        dbg!(&sk.as_bytes());

        let token = local::encrypt(&sk, &claims, None, Some(b"implicit assertion"))?;

        dbg!(&token);

        // Decide how we want to validate the claims after verifying the token itself.
        // The default verifies the `nbf`, `iat` and `exp` claims. `nbf` and `iat` are
        // always expected to be present.
        // NOTE: Custom claims, defined through `add_additional()`, are not validated.
        // This must be done manually.
        let mut validation_rules = ClaimsValidationRules::new();
        validation_rules.allow_non_expiring();
        let untrusted_token = UntrustedToken::<Local, V4>::try_from(&token)?;
        let trusted_token = local::decrypt(
            &sk,
            &untrusted_token,
            &validation_rules,
            None,
            Some(b"implicit assertion"),
        )?;
        assert_eq!(&claims, trusted_token.payload_claims().unwrap());

        let claims = trusted_token.payload_claims().unwrap();

        println!("{:?}", claims.get_claim("data"));
        println!("{:?}", claims.get_claim("iat"));

        Ok(())
    }
}
