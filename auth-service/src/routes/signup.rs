use serde::{Serialize, Deserialize};
use axum::{response::IntoResponse, Json,http::StatusCode, extract::State};

use crate::{app_state::AppState, domain::{UserStoreError,AuthAPIErrors, User, Email, Password}};

#[derive(Serialize, Deserialize,Debug)]
pub struct SignUp {
    pub email: String,
    pub password: String,
    #[serde(rename(deserialize="requires2FA"))]
    pub requires_2fa: bool
}


pub async fn signup(State(app_state): State<AppState>, Json(request): Json<SignUp>)-> impl IntoResponse 
{
    // Your signup logic here

    
    let email = 
        Email::parse(request.email)
            .map_err(|_| AuthAPIErrors::InvalidCredentials.into_response());

    let email = match email {
        Err(e) => return e,
        Ok(email) => email,
    };
    let password = 
        Password::parse(request.password)
            .map_err(|_| AuthAPIErrors::InvalidCredentials.into_response());
    let password = match password {
        Err(e) => return e,
        Ok(password) => password,
    };
    let requires_2fa = request.requires_2fa;





    let user = User::new(
        email.clone(),
        password.clone(),
        requires_2fa
    );

    let mut user_store = app_state.user_store.write().await;

    let returned_code = user_store.add_user(user).await;

    match returned_code {
        Ok(_) => {
            let response = Json(SignupResponse {
                message: format!("User {} created successfully", email.as_ref())
            });

            // If all checks pass, create the user (placeholder)
            // create_user(email, password, requires_2fa).await;

            return (StatusCode::CREATED,response).into_response()  
        },
        Err(e) => match e {
            UserStoreError::UserAlreadyExists => return AuthAPIErrors::UserAlreadyExists.into_response(),
            UserStoreError::InvalidCredentials => return AuthAPIErrors::InvalidCredentials.into_response(),
            _ => return AuthAPIErrors::InternalServerError.into_response(),
            }
    }

}



#[derive(Serialize,Deserialize,PartialEq,Debug)]
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