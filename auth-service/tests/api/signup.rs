//use crate::helpers::test_app;
use crate::helpers::get_random_email;
use crate::helpers::TestApp;
use auth_service::routes::signup::SignupResponse;
use auth_service::ErrorResponse;

/* *
#[tokio::test]
async fn should_return_201_if_valid_input() {

    let random_email = get_random_email();
    let test_case =
        serde_json::json!({
            "email": random_email,
            "password": "password123@",
            "requires2FA": true
        });

    //let app = test_app().await;
    let app = TestApp::new().await;

    let response = app.post_signup(&test_case).await;
    println!("signup:{}", response.status());
    println!("email:{}", test_case["email"]);
    println!("requires2FA:{}", test_case["requires2FA"]);
    println!("password:{}", test_case["password"]);

    

    let email_regex = fancy_regex::Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
    let password_regex = fancy_regex::Regex::new(r"^(?!.*\s)(?=.*[0-9])(?=.*[!@#$%^&*])(?=.{8,})").unwrap();

    // Q: how do I transform a Value into a string
        // A: use the to_string() method
    println!("{}", email_regex.is_match(&test_case["email"].as_str().unwrap()).unwrap());
    println!("{}", password_regex.is_match(&test_case["password"].as_str().unwrap()).unwrap());


    assert_eq!(response.status().as_u16(), 201);
}
* */

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
        let response = app.post_signup(&test_case).await;
        assert_eq!(response.status().as_u16(), 422,"Failed to validate signup for: {}", test_case);
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    
    let expected_response = SignupResponse {
        message: format!("User {} created successfully", random_email)
    };

    let test_case =
        serde_json::json!({
            "email": random_email,
            "password": "password123@",
            "requires2FA": false
        });

    let response = app.post_signup(&test_case).await;

    assert_eq!(response.status().as_u16(), 201);
    assert_eq!(response.json::<SignupResponse>()
        .await
        .expect("Could not deserialize response body to UserBody"),
        expected_response);
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

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
        let response = app.post_signup(&test_case).await;

        assert_eq!(response.status().as_u16(), 400, "Failed to validate signup for: {}", test_case);
        assert_eq!(response.json::<ErrorResponse>().await
            .expect("Could not deserialize response body to ErrorResponse")
            .error, "Invalid credentials".to_string());
    }

}

#[tokio::test]
async fn should_return_409_if_user_already_exists() {
    let app = TestApp::new().await;
    let random_email = get_random_email();

    let test_case =
        serde_json::json!({
            "email": random_email,
            "password": "password123@",
            "requires2FA": true
        });

    // First signup should succeed
    let response1 = app.post_signup(&test_case).await;
    assert_eq!(response1.status().as_u16(), 201);

    // Second signup with the same email should fail with 409
    let response2 = app.post_signup(&test_case).await;
    assert_eq!(response2.status().as_u16(), 409);

    assert_eq!(response2.json::<ErrorResponse>().await
        .expect("Could not deserialize response body to ErrorResponse")
        .error, "User already exists".to_string());
}   

