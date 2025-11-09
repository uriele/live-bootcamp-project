use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::utils::auth::generate_auth_cookie;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Verify2FARequest {
    pub email: String,

    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // Your 2FA verification logic here

    let email = request.email;
    let login_attempt_id = request.login_attempt_id;
    let two_fa_code = request.two_fa_code;

    let email = match Email::parse(email) {
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
        Ok(email) => email,
    };
    let login_attempt_id = match LoginAttemptId::parse(login_attempt_id) {
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
        Ok(login_attempt_id) => login_attempt_id,
    };

    let two_fa_code = match TwoFACode::parse(two_fa_code) {
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
        Ok(two_fa_code) => two_fa_code,
    };

    let code_tuple = match state.two_fa_code_store.read().await.get_code(&email).await {
        Err(_) => return (jar, Err(AuthAPIError::InvalidToken)),
        Ok(data) => data,
    };

    if (login_attempt_id, two_fa_code) != code_tuple {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    }

    let auth_cookie = generate_auth_cookie(&email);

    match auth_cookie {
        Ok(cookie) => {
            let updated_jar = jar.add(cookie);
            match state
                .two_fa_code_store
                .write()
                .await
                .remove_code(&email)
                .await
            {
                Err(_) => return (updated_jar, Err(AuthAPIError::InvalidToken)),
                Ok(_) => (),
            }

            let response = Verify2FAResponse {
                message: format!("User {} logged in successfully", email.as_ref()),
            };
            (updated_jar, Ok(response))
        }
        _ => (jar, Err(AuthAPIError::InternalServerError)),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Verify2FAResponse {
    pub message: String,
}
// If a user requires 2FA, this JSON body should be returned!
impl IntoResponse for Verify2FAResponse {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap();
        axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(body))
            .unwrap()
    }
}
