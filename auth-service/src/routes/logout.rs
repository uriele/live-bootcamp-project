

use axum_extra::extract::CookieJar;
use axum::{response::IntoResponse, http::StatusCode};
use crate::{
    domain::AuthAPIErrors,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME}
};

pub async fn logout(jar:CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthAPIErrors>) {
    // retrieve JWT cookie from CookieJar 

    let cookie = jar.get(JWT_COOKIE_NAME);

    // return AuthAPIErrors::MissingToken if cookie is not found
    let cookie = match cookie {
        Some(cookie) => cookie,
        None => return (jar, Err(AuthAPIErrors::MissingToken)),
    };

    let token = cookie.clone().value().to_owned();


    // only return AuthAPIErrors::InvalidToken if token is invalid
    match validate_token(&token).await {
        Err(_) => return (jar, Err(AuthAPIErrors::InvalidToken)),
        _ => (),   
    }

    // remove cookie before returning jar

    let updated_jar = jar.remove(JWT_COOKIE_NAME);

    (updated_jar, Ok(StatusCode::OK))

}