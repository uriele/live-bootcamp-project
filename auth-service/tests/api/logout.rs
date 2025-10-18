use crate::helpers::TestApp;
#[tokio::test]
async fn logout_return_auth_ui() { 
    let app = TestApp::new().await;
    let response = app.post_logout().await;
    println!("logout:{}", response.status());
    assert_eq!(response.status().as_u16(), 200);
}