use axum_extra::extract::cookie::{Cookie,SameSite};
use chrono::{Utc}; 
use super::constants::{JWT_COOKIE_NAME, JWT_SECRET};

use jsonwebtoken::{encode,decode,DecodingKey,EncodingKey,Validation};

use serde::{Serialize,Deserialize};

use crate::domain::email::Email;


pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>, GenerateTokenError> {
    let token = generate_auth_token(email)?;
    Ok(create_auth_cookie(token))
}


fn create_auth_cookie(token: String) -> Cookie<'static> {
    let cookie = Cookie::build((JWT_COOKIE_NAME, token))
        .path("/")
        .http_only(true)  // prevent JavaScript access the token
        .same_site(SameSite::Lax)// send cookie with "same-site" requests, and with "cross-site" top-level navigations
        .build();
    cookie
}

#[derive(Debug)]
pub enum GenerateTokenError {
    TokenError(jsonwebtoken::errors::Error),
    UnexpectedError,
}


pub const TOKEN_TTL_SECONDS: i64 = 600; // 10 minutes



pub fn generate_auth_token(email: &Email) -> Result<String, GenerateTokenError> {
    let delta =chrono::Duration::try_seconds(TOKEN_TTL_SECONDS)
        .ok_or(GenerateTokenError::UnexpectedError)?;

    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::UnexpectedError)?
        .timestamp();

    let exp: usize = exp 
        .try_into()
        .map_err(|_| GenerateTokenError::UnexpectedError)?;

    let sub = email.as_ref().to_owned();

    let claims = Claims{ sub, exp };

    create_token(&claims).map_err(GenerateTokenError::TokenError)
    
}



pub async fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let claim = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::default(),
    ).map(|data| data.claims);

    claim   
}


fn create_token(claims: &Claims) -> Result<String, jsonwebtoken::errors::Error> {
    let token = encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    ); 
    token

}


// jsonwebtoken claims structure needs to be serializable and deserializable
// and to include debug
#[derive(Serialize, Deserialize,Debug)]
pub struct Claims {
    pub sub: String,// Optional. Subject (whom token refers to)
    //aud: String,         // Optional. Audience
    exp: usize,  // // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    //iat: usize,          // Optional. Issued at (as UTC timestamp)
    //iss: String,         // Optional. Issuer
    //nbf: usize,          // Optional. Not Before (as UTC timestamp)
    //sub: String,         // Optional. Subject (whom token refers to)
}

#[cfg(test)]
mod tests{

    use super::*;
    //use crate::domain::email::Email;

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email = Email::parse("test@example.com".to_owned()).unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(),3); // JWTs have three parts separated by dots
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie(){
        let token = "test_token".to_string();
        let cookie = create_auth_cookie(token.clone());
        assert_eq!(cookie.name(), JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let email = Email::parse("test@example.com".to_owned()).unwrap();
        let token = generate_auth_token(&email).unwrap();
        let result = validate_token(&token).await.unwrap();
        assert_eq!(result.sub, email.as_ref());

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9)
                .expect("valid duration"))
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
}