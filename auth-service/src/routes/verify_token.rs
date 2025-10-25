use axum::response::IntoResponse;
use axum::http::StatusCode;
use axum::extract::State;
use std::ops::Deref;
use axum::{Json};
use serde::{Serialize, Deserialize};
use crate::{
    app_state::AppState,
    domain::AuthAPIErrors,
    utils::{auth::validate_token}
};


pub async fn verify_token(State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>) -> Result<impl IntoResponse, AuthAPIErrors> {
        
    match state.banned_token_store.read().await.is_token_banned(&request).await {
        Ok(val) => {
            if val {
                return Err(AuthAPIErrors::InvalidToken);
            }
        },
        Err(_) => return Err(AuthAPIErrors::InternalServerError),
    }

    match validate_token(&request).await {
        Err(_) =>  Err(AuthAPIErrors::InvalidToken),
        Ok(_) => Ok(StatusCode::OK)
        }

}


#[derive(Serialize, Deserialize,Debug,Clone)]
pub struct VerifyTokenRequest {
    token: String,
}

impl Deref for VerifyTokenRequest {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.token
    }
}