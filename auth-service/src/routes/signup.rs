use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use secrecy::{ExposeSecret, Secret};
use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password, User},//data_stores::UserStoreError},
};

#[derive(Deserialize, Debug)]
pub struct SignupRequest {
    pub email: Secret<String>,
    pub password: Secret<String>,
    #[serde(rename(deserialize = "requires2FA"))]
    pub requires_2fa: bool,
}


#[tracing::instrument(name = "Signup", skip_all)] // New!
pub async fn signup(
    State(app_state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse,AuthAPIError> {
    // Your signup logic here

    let email =
        Email::parse(request.email).map_err(|_| AuthAPIError::InvalidCredentials.into_response());

    let email = match email {
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
        Ok(email) => email,
    };
    let password = Password::parse(request.password)
        .map_err(|_| AuthAPIError::InvalidCredentials.into_response());
    let password = match password {
        Err(_) => return Err(AuthAPIError::InvalidCredentials),
        Ok(password) => password,
    };
    let requires_2fa = request.requires_2fa;

    let user = User::new(email.clone(), password.clone(), requires_2fa);

    let mut user_store = app_state.user_store.write().await;

    let returned_code = user_store.add_user(user).await;

    match returned_code {
        Ok(_) => {
            let response = Json(SignupResponse {
                message: format!("User {} created successfully", email.as_ref().expose_secret()),
            });

            // If all checks pass, create the user (placeholder)
            // create_user(email, password, requires_2fa).await;

            return Ok((StatusCode::CREATED, response).into_response());
        },

        Err(e) => {
             return Err(AuthAPIError::UserAlreadyExists); //UnexpectedError(e.into())); // Updated!
        }
    
        /*
        Err(e) =>  match e {
            UserStoreError::UserAlreadyExists => {
                return Err(AuthAPIError::UserAlreadyExists)
            }
            UserStoreError::InvalidCredentials => {
                return Err(AuthAPIError::InvalidCredentials)
            }
            _ => return Err(AuthAPIError::InternalServerError),
        },
        */
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SignupResponse {
    pub message: String,
}

impl IntoResponse for SignupResponse {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap();
        axum::response::Response::builder()
            .status(StatusCode::CREATED)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(body))
            .unwrap()
    }
}
