use crate::helpers::TestApp;

#[tokio::test]
async fn login_return_auth_ui() { 
    let app = TestApp::new().await;
    let response = app.post_login().await;
    println!("login:{}", response.status());
    assert_eq!(response.status().as_u16(), 200);
}