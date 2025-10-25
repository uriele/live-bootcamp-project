use crate::helpers::TestApp;
//use auth_service::routes::login::LoginResponse;
//use auth_service::ErrorResponse;


use auth_service::{utils::constants::JWT_COOKIE_NAME};

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    // Call the log-in route with valid credentials and assert that a
    // 200 HTTP status code is returned along with a JWT cookie in the response.

    let app = TestApp::new().await;

    let random_email = crate::helpers::get_random_email();
    let password = "password123@".to_string();

    let signup_body =
        serde_json::json!({
            "email": random_email.clone(),
            "password": password.clone(),
            "requires2FA": false
        });

    // First signup should succeed
    let response = app.post_signup(&signup_body).await;
    assert_eq!(response.status().as_u16(), 201);
    let login_body =
        serde_json::json!({
            "email": random_email.clone(),
            "password": password.clone(),
        });

    let response = app.post_login(&login_body).await;
    
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found in response");

    assert!(!auth_cookie.value().is_empty(), "Auth cookie is empty");
}


#[tokio::test]
async fn should_return_400_if_invalid_input() {
    // Call the log-in route with invalid credentials and assert that a
    // 400 HTTP status code is returned along with the appropriate error message. 
    let app = TestApp::new().await;

    let random_email = crate::helpers::get_random_email();
    let test_cases = [
        serde_json::json!({
            "email": random_email,
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": random_email,
            "password": "pass@12",
            "requires2FA": false
        }),
        serde_json::json!({
            "email": "",
            "password": "password123@",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "email",
            "password": "password123@",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "email@email",
            "password": "password123@",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "email.email",
            "password": "password123@",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "@email.email",
            "password": "password123@",
            "requires2FA": true
        })
    ];

    for test_case in test_cases {
        let response = app.post_login(&test_case).await;
        assert_eq!(response.status().as_u16(), 400,"Failed to validate login for: {}", test_case);
    }
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {

    let app = TestApp::new().await;
    
    let random_email = crate::helpers::get_random_email();

    let test_zero =
        serde_json::json!({
            "email": random_email,
            "password": "password123@",
            "requires2FA": true
        });

    // First signup should succeed
    let response1 = app.post_signup(&test_zero).await;
    assert_eq!(response1.status().as_u16(), 201);


    let response1 = app.post_login(&test_zero).await;
    assert_eq!(response1.status().as_u16(), 200, "Login successful for: {}", test_zero);



    let test_cases =[
        serde_json::json!({
            "email": "valid.email@example.com",
            "password": "validPassword123@"
        }),
        serde_json::json!({
            "email": random_email,
            "password": "invalidPassword123@"
        }),
    ];

    for test_case in test_cases {
        let response = app.post_login(&test_case).await;
        assert_eq!(response.status().as_u16(), 401,"Failed to validate login for: {}", test_case);  
    }
}
