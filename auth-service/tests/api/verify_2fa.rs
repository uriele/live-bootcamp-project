
use crate::helpers::{TestApp};
use auth_service::domain::{LoginAttemptId, TwoFACode,Email};
use fake::{Fake, faker::internet::en::FreeEmail};

fn test_login() -> String{
    LoginAttemptId::default().as_ref().to_owned()
}

fn test_2fa()-> String{
    TwoFACode::default().as_ref().to_owned()
}

fn test_email() -> String{
    FreeEmail().fake()
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
       let two_fa_requests = [serde_json::json!({
        "email": "not-an-email",
        "loginAttempId": test_login(),
        "2FACode": test_2fa()
    }),
    serde_json::json!({
        "mail": test_email(),
        "loginAttemptId": "not-a-uuid",
        "FACode": test_2fa()
    }),
    serde_json::json!({
        "email": test_email(),
        "loginttemptId": test_login(),
        "2FACoe": "not-a-6-digit-code"
    })];

    for two_fa_request in two_fa_requests{
        let app = TestApp::new().await;
        let response = app.post_verify_2fa(&two_fa_request).await;
        assert_eq!(response.status(), 422);
    }


}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let two_fa_requests = [serde_json::json!({
        "email": "not-an-email",
        "loginAttemptId": test_login(),
        "2FACode": test_2fa()
    }),
    serde_json::json!({
        "email": test_email(),
        "loginAttemptId": "not-a-uuid",
        "2FACode": test_2fa()
    }),
    serde_json::json!({
        "email": test_email(),
        "loginAttemptId": test_login(),
        "2FACode": "not-a-6-digit-code"
    })];

    for two_fa_request in two_fa_requests{
        let response = app.post_verify_2fa(&two_fa_request).await;
        assert_eq!(response.status(), 400);
    }

}

#[tokio::test]
async fn should_return_200_if_valid_2fa_code() {
    let app = TestApp::new().await;
    let email: String = FreeEmail().fake();
    let password: String = "P@ssword123!".to_string();

    let request_signup = serde_json::json!({"email": email.clone(),
    "password": password.clone(),
    "requires2FA": true});

    let signup_request =  app.post_signup(&request_signup).await;
    assert_eq!(signup_request.status(),201);

    let request_login=serde_json::json!({
        "email": email.clone(),
        "password": password.clone()
    });
    let login_request= app.post_login(&request_login).await;
    assert_eq!(login_request.status(),206);


    let (login_attempt_id, two_fa_code) = app.two_fa_code_store.read().await.get_code(&Email::parse(email.clone()).unwrap()).await.unwrap();

    
    let two_fa_request = serde_json::json!({
        "email": email,
        "loginAttemptId": login_attempt_id.as_ref().to_string(),
        "2FACode": two_fa_code.as_ref().to_string()
    });
    
    let response = app.post_verify_2fa(&two_fa_request).await;

    println!("{:?}",response);
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn should_return_401_if_same_code_twice(){
     let app = TestApp::new().await;
    let email: String = FreeEmail().fake();
    let password: String = "P@ssword123!".to_string();

    let request_signup = serde_json::json!({"email": email.clone(),
    "password": password.clone(),
    "requires2FA": true});

    let signup_request =  app.post_signup(&request_signup).await;
    assert_eq!(signup_request.status(),201);

    let request_login=serde_json::json!({
        "email": email.clone(),
        "password": password.clone()
    });
    let login_request= app.post_login(&request_login).await;
    assert_eq!(login_request.status(),206);


    let (login_attempt_id, two_fa_code) = app.two_fa_code_store.read().await.get_code(&Email::parse(email.clone()).unwrap()).await.unwrap();

    
    let two_fa_request = serde_json::json!({
        "email": email,
        "loginAttemptId": login_attempt_id.as_ref().to_string(),
        "2FACode": two_fa_code.as_ref().to_string()
    });
    
    let response = app.post_verify_2fa(&two_fa_request).await;

    assert_eq!(response.status(), 200);

    let response = app.post_verify_2fa(&two_fa_request).await;  
    println!("{:?}",response);
    assert_eq!(response.status(), 401);
}