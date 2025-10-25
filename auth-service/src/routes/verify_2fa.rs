use serde::{Serialize, Deserialize};
use axum::{response::{IntoResponse,Json},
    http::StatusCode,
    extract::State
};
use crate::{app_state::AppState};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)] 
pub struct TwoFactorAuthRequest {
    pub email: String,

    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String
}


pub async fn verify_2fa(State(_state): State<AppState>,
    Json(_request): Json<TwoFactorAuthRequest>) -> impl IntoResponse {
    // Your 2FA verification logic here

    // For demonstration, we'll just return OK
    StatusCode::OK
}