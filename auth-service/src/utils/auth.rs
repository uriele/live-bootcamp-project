use super::constants::{JWT_COOKIE_NAME, JWT_SECRET};
use crate::app_state::{AppState, BannedTokenStoreType};
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use color_eyre::eyre::{eyre, Context, ContextCompat, Result};
use serde::{Deserialize, Serialize};
use secrecy::{ExposeSecret, Secret};

use crate::domain::email::Email;
use crate::domain::AuthAPIError;
use axum_extra::extract::CookieJar;

#[tracing::instrument(name = "Generate Auth Cookie", skip_all)]
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>> {
    let token = generate_auth_token(email)?;
    Ok(create_auth_cookie(token))
}

#[tracing::instrument(name = "Create Auth Cookie", skip_all)]
fn create_auth_cookie(token: Secret<String>) -> Cookie<'static> {
    let cookie = Cookie::build((JWT_COOKIE_NAME, token.expose_secret().clone()))
        .path("/")
        .http_only(true) // prevent JavaScript access the token
        .same_site(SameSite::Lax) // send cookie with "same-site" requests, and with "cross-site" top-level navigations
        .build();
    cookie
}

#[derive(Debug)]
pub enum GenerateTokenError {
    TokenError(jsonwebtoken::errors::Error),
    UnexpectedError,
}

pub const TOKEN_TTL_SECONDS: i64 = 600; // 10 minutes


#[tracing::instrument(name = "Generate Auth Token", skip_all)]
pub fn generate_auth_token(email: &Email) -> Result<Secret<String>> {
    let delta = chrono::Duration::try_seconds(TOKEN_TTL_SECONDS)
        .wrap_err("Failed to create 10 minute time delta")?;
        

    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(eyre!("failed to add 10 minutes to current time"))?
        .timestamp();

    let exp: usize = exp.try_into().wrap_err(format!(
        "failed to cast exp time to usize. exp time: {}",
        exp
    ))?;

    let sub = email.as_ref().expose_secret().to_owned();

    let claims = Claims { sub, exp };

    create_token(&claims)
}


#[tracing::instrument(name = "Validate Token", skip_all)]
pub async fn validate_token(
    token: &Secret<String>,
    banned_token_store: BannedTokenStoreType,
) -> Result<Claims> {
    match banned_token_store.read().await.is_token_banned(token).await {
        Ok(value) => {
            if value {
                return Err(eyre!("token is banned"));
            }
        }
        Err(e) => return Err(e.into()),
    }

    decode::<Claims>(
        token.expose_secret(),
        &DecodingKey::from_secret(JWT_SECRET.expose_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .wrap_err("failed to decode token")
}


#[tracing::instrument(name = "Create Token", skip_all)]
fn create_token(claims: &Claims) -> Result<Secret<String>> {
    let token=encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.expose_secret().as_bytes()),
    )
    .wrap_err("failed to create token")?;
    Ok(Secret::new(token))
}

#[tracing::instrument(name = "Check for Token Validity", skip_all)]
pub async fn check_for_token_validity(
    state: AppState,
    jar: &CookieJar,
) -> Result<(), AuthAPIError> {
    let cookie = jar.get(JWT_COOKIE_NAME);

    // return AuthAPIError::MissingToken if cookie is not found
    let cookie = match cookie {
        Some(cookie) => cookie,
        None => return Err(AuthAPIError::MissingToken),
    };

    let token = Secret::new(cookie.clone().value().to_owned());

    // only return AuthAPIError::InvalidToken if token is invalid
    validate_token(&token,state.banned_token_store.clone())
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    state
        .banned_token_store
        .write()
        .await
        .ban_token(token)
        .await
        .map_err(|_| AuthAPIError::InternalServerError)?;
    Ok(())
}

// jsonwebtoken claims structure needs to be serializable and deserializable
// and to include debug
#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: String, // Optional. Subject (whom token refers to)
    //aud: String,         // Optional. Audience
    exp: usize, // // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
                //iat: usize,          // Optional. Issued at (as UTC timestamp)
                //iss: String,         // Optional. Issuer
                //nbf: usize,          // Optional. Not Before (as UTC timestamp)
                //sub: String,         // Optional. Subject (whom token refers to)
}

#[cfg(test)]
mod tests {

    use super::*;
    //use crate::domain::email::Email;

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email = Email::parse("test@example.com".to_owned().into()).unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3); // JWTs have three parts separated by dots
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        let token = Secret::new("test_token".to_string());
        let cookie = create_auth_cookie(token.clone());
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(&cookie.value(), token.expose_secret());
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    /*
    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let email = Email::parse("test@example.com".to_owned()).unwrap();
        let token = generate_auth_token(&email).unwrap();
        let result = validate_token(&token).await.unwrap();
        assert_eq!(result.sub, email.as_ref());

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp() as usize;
        assert!(result.exp > exp);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let token = "invalid_token".to_string();
        let result = validate_token(&token).await;
        assert!(result.is_err());
    }
    */
}
