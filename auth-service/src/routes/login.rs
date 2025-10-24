use axum::{Json};
use serde::{Serialize, Deserialize};
use axum::{response::IntoResponse, http::StatusCode, extract::State};

use crate::{app_state::AppState, domain::{AuthAPIErrors, Email, Password}, utils::auth};


use axum_extra::extract::{cookie::Cookie, CookieJar};


use crate::{utils::auth::generate_auth_cookie};

//use crate::make_response_functions;
//make_response_functions!(login);

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}


pub async fn login<T>(
    State(app_state): State<AppState<T>>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>
) -> (CookieJar,Result<impl IntoResponse,AuthAPIErrors>) 
where T: crate::domain::UserStore + Send + Sync + 'static +Clone {
    // Your login logic here
    // For example, validate credentials, generate tokens, etc.
    let email = 
        Email::parse(request.email);

    let email = match email {
        Ok(email) => email,
        _ => return (jar, Err(AuthAPIErrors::InvalidCredentials)),
    };

    let password = 
        Password::parse(request.password);
    let password = match password {
        Ok(password) => password,
        _ => return (jar, Err(AuthAPIErrors::InvalidCredentials)),
    };

    // Placeholder logic for user authentication
    let user_store = app_state.user_store.read().await;
    let valid_user = user_store.validate_credentials(email.clone(), password.clone()).await;

    match valid_user {
        Ok(is_valid) => {
            if is_valid {
                println!("User {:?} logged in successfully", &email);
                println!("Password: {:?}", &password);
                println!("User store: {:?}", user_store.get_user(email.clone()).await);
            } else {
                return (jar, Err(AuthAPIErrors::WrongEmailOrPassword))
            }
        },
        _ => return (jar, Err(AuthAPIErrors::WrongEmailOrPassword)),
    };

    let auth_cookie = 
    generate_auth_cookie(&email);

    let auth_cookie = match auth_cookie {
        Ok(cookie) => cookie,
        _ => return (jar, Err(AuthAPIErrors::InternalServerError)),
    };




    let updated_jar = jar.add(auth_cookie);

    (updated_jar, Ok(StatusCode::OK.into_response()))
}

#[derive(Serialize,Deserialize,PartialEq,Debug)]
pub struct LoginResponse {
    pub message: String,
}

impl IntoResponse for LoginResponse {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap();
        axum::response::Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(body))
            .unwrap()
    }
}