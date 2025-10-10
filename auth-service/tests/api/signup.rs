use crate::helpers::test_app;
use crate::helpers::get_random_email;
#[tokio::test]
async fn signup_return_auth_ui() { 

    let random_email = get_random_email();
    let test_case =
        serde_json::json!({
            "email": random_email,
            "password": "password123@",
            "requires2FA": true
        });

    let app = test_app().await;

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

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = test_app().await;
    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "email": random_email,
            "password": "passwo",  // invalid password format
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "invalid-email",  // invalid email format
            "password": "short1234@",
            "requires2FA": false
        })
    ];

    for test_case in test_cases {
        let response = app.post_signup(&test_case).await;
        assert_eq!(response.status().as_u16(), 422,"Failed to validate signup for: {}", test_case);
    }
}