use crate::{app_state::AppState, domain::AuthAPIError, utils::auth::validate_token};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize};
use std::ops::Deref;
use secrecy::Secret;
#[tracing::instrument(name = "Verify Token", skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    
    match validate_token(&request, state.banned_token_store.clone()).await {
        Err(_) => Err(AuthAPIError::InvalidToken),
        Ok(_) => Ok(StatusCode::OK),
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct VerifyTokenRequest {
    token: Secret<String>,
}

impl Deref for VerifyTokenRequest {
    type Target = Secret<String>;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}
