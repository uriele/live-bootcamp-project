use crate::helpers::test_app;
#[tokio::test]
async fn verify_token_return_auth_ui() { 
    let app = test_app().await;
    let response = app.post_verify_token().await;
    println!("verify_token:{}", response.status());
    assert_eq!(response.status().as_u16(), 200);
}