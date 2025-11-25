use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},
};
use secrecy::{ExposeSecret, Secret};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::utils::auth::generate_auth_cookie;

#[derive(Deserialize, Debug, Clone)]
pub struct Verify2FARequest {
    pub email: Secret<String>,

    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: Secret<String>,
    #[serde(rename = "2FACode")]
    pub two_fa_code: Secret<String>,
}


#[tracing::instrument(name = "Verify 2FA", skip_all)]
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
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e.into()))),
        Ok(email) => email,
    };
    let login_attempt_id = match LoginAttemptId::parse(login_attempt_id) {
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e.into()))),
        Ok(login_attempt_id) => login_attempt_id,
    };

    let two_fa_code = match TwoFACode::parse(two_fa_code) {
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e.into()))),
        Ok(two_fa_code) => two_fa_code,
    };

    let code_tuple = match state.two_fa_code_store.read().await.get_code(&email).await {
        Err(e) => return (jar, Err(AuthAPIError::UnexpectedError(e.into()))),
        Ok(data) => data,
    };

    if (login_attempt_id, two_fa_code) != code_tuple {
        return (jar, Err(AuthAPIError::InvalidCredentials)); //UnexpectedError(eyre!("Invalid Credential"))));
    }

    let auth_cookie = generate_auth_cookie(&email);

    match auth_cookie {
        Ok(cookie) => {
            let updated_jar = jar.clone().add(cookie);
            if let Err(_) = state
                .two_fa_code_store
                .write()
                .await
                .remove_code(&email)
                .await
            {
                return (jar, Err(AuthAPIError::InvalidToken));
            }
            

            let response = Verify2FAResponse {
                message: format!("User {} logged in successfully", email.as_ref().expose_secret()),
            };
            (updated_jar, Ok(response))
        }
        Err(e) => (jar, Err(AuthAPIError::UnexpectedError(e.into()))),
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
