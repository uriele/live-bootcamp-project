use serde::{Serialize, Deserialize};
use axum::{response::IntoResponse, Json,http::StatusCode, extract::State};

use crate::{app_state::AppState, domain::{UserStoreError,AuthAPIErrors, User}};

//use crate::make_response_functions;
//make_response_functions!(signup);

#[derive(Serialize, Deserialize)]
pub struct SignUp {
    email: String,
    password: String,
    #[serde(rename(deserialize="requires2FA"))]
    requires_2fa: bool
}



pub async fn signup<T>(State(app_state): State<AppState<T>>, Json(payload): Json<SignUp>)-> impl IntoResponse 
where T: crate::domain::UserStore + Send + Sync + 'static +Clone {
    // Your signup logic here
    let email = payload.email;
    let password = payload.password;
    let requires_2fa = payload.requires_2fa;

    let email_regex = fancy_regex::Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
    let password_regex = fancy_regex::Regex::new(r"^(?!.*\s)(?=.*[0-9])(?=.*[!@#$%^&*])(?=.{8,})").unwrap();


    if !email_regex.is_match(&email.as_str()).unwrap() {
        return AuthAPIErrors::InvalidCredentials.into_response();
    }

    //password at least 8 letters and a number and a symbol, no spaces
    if !password_regex.is_match(&password.as_str()).unwrap() {
        return AuthAPIErrors::InvalidCredentials.into_response();
    }


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
                message: format!("User {} created successfully", email)
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