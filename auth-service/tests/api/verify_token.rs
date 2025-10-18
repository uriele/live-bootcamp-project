use crate::helpers::TestApp;
#[tokio::test]
async fn verify_token_return_auth_ui() { 
    let app = TestApp::new().await;
    let response = app.post_verify_token().await;
    println!("verify_token:{}", response.status());
    assert_eq!(response.status().as_u16(), 200);
}