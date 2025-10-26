use axum::{Json};
use serde::{Serialize, Deserialize};
use axum::{response::IntoResponse, http::StatusCode, extract::State};

use crate::{app_state::AppState, domain::{AuthAPIError, Email, Password}};

use crate::domain::data_stores::{LoginAttemptId,TwoFACode};
use axum_extra::extract::{CookieJar};


use crate::{utils::auth::generate_auth_cookie};


#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}


pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>)
{
    // Your login logic here
    // For example, validate credentials, generate tokens, etc.
    let email = 
        Email::parse(request.email);

    let email = match email {
        Ok(email) => email,
        _ => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let password = 
        Password::parse(request.password);
    let password = match password {
        Ok(password) => password,
        _ => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    // Placeholder logic for user authentication
    let user_store = state.user_store.read().await;
    let user = match user_store.get_user(email.clone()).await{
        Ok(user) => user,
        _ => return (jar, Err(AuthAPIError::WrongEmailOrPassword)),
    };

    let is_valid = match user_store.validate_credentials(email.clone(), password.clone()).await {
        Ok(valid) => valid,
        _ => return (jar, Err(AuthAPIError::WrongEmailOrPassword)),
    };

    if !is_valid {
        return (jar, Err(AuthAPIError::WrongEmailOrPassword));
    }

    match user.requires_2fa {
        true => handle_2fa_login(&email, &state, jar).await,
        false => handle_standard_login(&email, jar).await,
    }
}


async fn handle_2fa_login(email: &Email, state: &AppState, jar: CookieJar) -> (CookieJar,Result<LoginResponse, AuthAPIError>) {

    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    match state.two_fa_code_store.write().await
        .add_code(
            email.clone(),
            login_attempt_id.clone(),
            two_fa_code.clone()
        ).await {
        Ok(_) => (),
        Err(_) => return (jar, Err(AuthAPIError::InternalServerError)),
    };



    // TODO: send 2FA code via the email client. Return `AuthAPIError::UnexpectedError` if the operation fails.
    match state.email_client.read()
        .await
        .send_email(email, 
        login_attempt_id.as_ref(),
        two_fa_code.as_ref()
        )
        .await {
        Ok(_) => (),
        Err(_) => return (jar, Err(AuthAPIError::InternalServerError)),
    };


    // Finally, we need to return the login attempt ID to the client
    let response = LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_owned(),
        login_attempt_id: login_attempt_id.as_ref().to_owned(),
    });

    (jar, Ok(response))
}

async fn handle_standard_login(email: &Email, jar: CookieJar) ->  (CookieJar,Result<LoginResponse, AuthAPIError>){
    let auth_cookie =  generate_auth_cookie(&email);

    match auth_cookie {
        Ok(cookie) => {
            let updated_jar = jar.add(cookie);
            let response = LoginResponse::RegularAuth(
                RegularAuthResponse {
                    message: format!("User {} logged in successfully", email.as_ref())
                }
            );
            (updated_jar, Ok(response))
        }
        _ => (jar, Err(AuthAPIError::InternalServerError)),
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth(RegularAuthResponse),
    TwoFactorAuth(TwoFactorAuthResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegularAuthResponse {
    pub message: String,
}
// If a user requires 2FA, this JSON body should be returned!
#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

impl IntoResponse for RegularAuthResponse {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap();
        axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(body))
            .unwrap()
    }
}

impl IntoResponse for TwoFactorAuthResponse {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap();
        axum::response::Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(body))
            .unwrap()
    }
}


impl IntoResponse for LoginResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            LoginResponse::RegularAuth(r) => r.into_response(),
            LoginResponse::TwoFactorAuth(t) => t.into_response(),
        }
    }
}
