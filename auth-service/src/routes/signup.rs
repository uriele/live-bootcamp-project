use serde::{Serialize, Deserialize};
use axum::{Json};

//use crate::make_response_functions;
//make_response_functions!(signup);

#[derive(Serialize, Deserialize)]
pub struct SignUp {
    email: String,
    password: String,
    #[serde(rename(deserialize="requires2FA"))]
    requires_2fa: bool
}



pub async fn signup(Json(payload): Json<SignUp>)  -> axum::http::StatusCode {
    // Your signup logic here
    let email = payload.email;
    let password = payload.password;
    //let _requires_2fa = payload.requires_2fa;

    let email_regex = fancy_regex::Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
    let password_regex = fancy_regex::Regex::new(r"^(?!.*\s)(?=.*[0-9])(?=.*[!@#$%^&*])(?=.{8,})").unwrap();


    if !email_regex.is_match(&email.as_str()).unwrap() {
        return axum::http::StatusCode::UNPROCESSABLE_ENTITY;
    }

    //password at least 8 letters and a number and a symbol, no spaces
    if !password_regex.is_match(&password.as_str()).unwrap() {
        return axum::http::StatusCode::UNPROCESSABLE_ENTITY;
    }

    // If all checks pass, create the user (placeholder)
    // create_user(email, password, requires_2fa).await;

    axum::http::StatusCode::CREATED
}