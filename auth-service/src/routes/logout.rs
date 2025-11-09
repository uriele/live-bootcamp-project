use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

use crate::{
    app_state::AppState,
    domain::AuthAPIError,
    utils::{auth::check_for_token_validity, constants::JWT_COOKIE_NAME},
};

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // retrieve JWT cookie from CookieJar

    match check_for_token_validity(state, &jar).await {
        Err(e) => (jar, Err(e)),
        Ok(_) => {
            let updated_jar = jar.remove(JWT_COOKIE_NAME);
            (updated_jar, Ok(StatusCode::OK))
        }
    }
}
