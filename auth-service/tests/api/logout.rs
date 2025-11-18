use crate::helpers::FakeJWT;
use crate::helpers::TestApp;
use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};
use fake::faker::internet::en::FreeEmail;
use fake::Fake;
use reqwest::Url;
use test_helpers::api_test;

#[api_test]
async fn should_return_400_if_jwt_cookie_missing() {
    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 400);

    let error_response: ErrorResponse = response
        .json()
        .await
        .expect("Failed to parse error response");

    assert_eq!(error_response.error, format!("Missing token"));
}

#[api_test]
async fn should_return_401_if_invalid_token() {
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

#[api_test]
async fn should_return_200_if_valid_jwt_cookie() {
    // add invalid cookie,
    // JWT is supposed to be in the form xxxxx.yyyyy.zzzz

    // create a vector from many fake JWT you can also use quickcheck if it's simpler

    for _ in 0..100 {
        let fake_jwt = FakeJWT::parse(FreeEmail().fake());
        app.cookie_jar.add_cookie_str(
            &format!(
                "{}={}; HttpOnly; SameSite=Lax; Secure; Path=/",
                JWT_COOKIE_NAME, *fake_jwt
            ),
            &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
        );
        let response = app.post_logout().await;
        assert_eq!(response.status().as_u16(), 200);

        let banned_token_store = app.banned_token_store.read().await;
        let contains_token = banned_token_store
            .is_token_banned(&fake_jwt)
            .await
            .expect("Failed to check if token is banned");

        println!(
            "Checking if token {} is banned: {}",
            *fake_jwt, contains_token
        );
        assert_eq!(contains_token, true);
    }
}

#[api_test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    for _ in 0..100 {
        let fake_jwt = FakeJWT::parse(FreeEmail().fake());
        app.cookie_jar.add_cookie_str(
            &format!(
                "{}={}; HttpOnly; SameSite=Lax; Secure; Path=/",
                JWT_COOKIE_NAME, *fake_jwt
            ),
            &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
        );
        let response = app.post_logout().await;
        assert_eq!(response.status().as_u16(), 200);
        let response = app.post_logout().await;
        assert_eq!(response.status().as_u16(), 400);
    }
}
