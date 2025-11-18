use crate::helpers::FakeJWT;
use crate::helpers::TestApp;
use fake::faker::internet::en::FreeEmail;
use fake::Fake;
use test_helpers::api_test;

use auth_service::utils::constants::JWT_COOKIE_NAME;

#[api_test]
async fn should_return_200_valid_token() {
    for _ in 0..125 {
        let token = FakeJWT::parse(FreeEmail().fake());
        let request_body = serde_json::json!({
            "token": token,
        });
        let response = app.post_verify_token(&request_body).await;
        assert_eq!(response.status().as_u16(), 200);
    }

    let myemail: String = FreeEmail().fake();
    let mypassword: String = "Password123!".into();

    let signup_body = serde_json::json!({
        "email": myemail.clone(),  // invalid email format
        "password": mypassword.clone(),  // invalid password format
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": myemail.clone(),
        "password": mypassword.clone()
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    let token = auth_cookie.value();

    let verify_token_body = serde_json::json!({
        "token": &token,
    });

    let response = app.post_verify_token(&verify_token_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[api_test]
async fn should_return_401_if_invalid_token() {
    let request_body = serde_json::json!({
        "token": "this.is.an.invalid.token",
    });
    let response = app.post_verify_token(&request_body).await;
    assert_eq!(response.status().as_u16(), 401);
}

#[api_test]
async fn should_return_422_if_malformed_input() {
    let test_cases = vec![
        serde_json::json!({
            "token": true,
        }),
        serde_json::json!({}),
    ];

    for test_case in test_cases {
        let response = app.post_verify_token(&test_case).await;
        assert_eq!(response.status().as_u16(), 422);
    }
}

#[api_test]
async fn should_return_401_if_banned_token() {
    let fake_jwt = FakeJWT::parse(FreeEmail().fake());

    // Ban the token
    {
        let fake_jwt = fake_jwt.to_string();
        let mut banned_token_store = app.banned_token_store.write().await;
        banned_token_store
            .ban_token(fake_jwt)
            .await
            .expect("Failed to ban token");
    }

    let request_body = serde_json::json!({
        "token": fake_jwt,
    });
    let response = app.post_verify_token(&request_body).await;
    assert_eq!(response.status().as_u16(), 401);
}
