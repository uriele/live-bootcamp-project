use crate::helpers::TestApp;
use crate::helpers::get_random_email;


#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "email": random_email,
            "password": "password@12",  // invalid password format
            "requires2FA": "true"
        }),
        serde_json::json!({
            "email": "invalid@email.com",  // invalid email format
            "password": 12345678,  // invalid password format
            "requires2FA": false
        }),
        
        serde_json::json!({
            "email": 1234,  // invalid email format
            "password": "short1234@",
            "requires2FA": true
        })
    ];

    for test_case in test_cases {
        let response = app.post_verify_token(&test_case).await;
        assert_eq!(response.status().as_u16(), 422,"Failed to validate verify_token for: {}", test_case);
    }
}