

use axum_extra::extract::CookieJar;
use axum::{response::IntoResponse, http::StatusCode};
use axum::extract::State;

use crate::{
    app_state::AppState,
    domain::AuthAPIErrors,
    utils::{auth::check_for_token_validity, constants::JWT_COOKIE_NAME}
};

pub async fn logout(State(_state): State<AppState>,
    jar: CookieJar) -> (CookieJar, Result<impl IntoResponse, AuthAPIErrors>) {
    // retrieve JWT cookie from CookieJar 

    match check_for_token_validity(&jar).await {
        Err(e) =>  (jar, Err(e)),
        Ok(_) => {
            let updated_jar = jar.remove(JWT_COOKIE_NAME);
            (updated_jar, Ok(StatusCode::OK))
        }
    }
}