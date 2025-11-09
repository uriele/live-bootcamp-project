use crate::{app_state::AppState, domain::AuthAPIError, utils::auth::validate_token};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    match state
        .banned_token_store
        .read()
        .await
        .is_token_banned(&request)
        .await
    {
        Ok(val) => {
            if val {
                return Err(AuthAPIError::InvalidToken);
            }
        }
        Err(_) => return Err(AuthAPIError::InternalServerError),
    }

    match validate_token(&request).await {
        Err(_) => Err(AuthAPIError::InvalidToken),
        Ok(_) => Ok(StatusCode::OK),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerifyTokenRequest {
    token: String,
}

impl Deref for VerifyTokenRequest {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}
