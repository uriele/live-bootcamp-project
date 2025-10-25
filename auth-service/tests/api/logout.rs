use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};
use fake::Fake;
use reqwest::Url;
use crate::helpers::TestApp;
use auth_service::domain::Email;
use fake::faker::internet::en::FreeEmail;
use auth_service::utils::auth::generate_auth_token;
use std::ops::Deref;
pub struct FakeJWT(String);

impl FakeJWT{
    pub fn parse(email:String) -> Self {
        Self(generate_auth_token(&Email::parse(email).unwrap()).unwrap())    
    }
}

impl Deref for FakeJWT {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 400);

    let error_response: ErrorResponse = response
        .json()
        .await
        .expect("Failed to parse error response");

    assert_eq!(
        error_response.error,
        format!("Missing token")
    );
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    let app = TestApp::new().await;

    // add invalid cookie, 
    // JWT is supposed to be in the form xxxxx.yyyyy.zzzz
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status().as_u16(), 401);
}






#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() {
    
    // add invalid cookie, 
    // JWT is supposed to be in the form xxxxx.yyyyy.zzzz

    // create a vector from many fake JWT you can also use quickcheck if it's simpler

    for _ in 0..100 {
        let fake_jwt = FakeJWT::parse(FreeEmail().fake());
        let app = TestApp::new().await;
        app.cookie_jar.add_cookie_str(
            &format!("{}={}; HttpOnly; SameSite=Lax; Secure; Path=/", JWT_COOKIE_NAME, *fake_jwt),
            &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
        );
        let response = app.post_logout().await;
        assert_eq!(response.status().as_u16(), 200);
    }
}


#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    for _ in 0..100 {
        let fake_jwt = FakeJWT::parse(FreeEmail().fake());
        let app = TestApp::new().await;
        app.cookie_jar.add_cookie_str(
            &format!("{}={}; HttpOnly; SameSite=Lax; Secure; Path=/", JWT_COOKIE_NAME, *fake_jwt),
            &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
        );
        let response = app.post_logout().await;
        assert_eq!(response.status().as_u16(), 200);
        let response = app.post_logout().await;
        assert_eq!(response.status().as_u16(), 400);
    }
}