use crate::helpers::test_app;
#[tokio::test]
async fn signup_return_auth_ui() { 
    let app = test_app().await;
    let response = app.post_signup().await;
    println!("signup:{}", response.status());
    assert_eq!(response.status().as_u16(), 200);
}