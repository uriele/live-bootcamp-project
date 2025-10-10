use crate::helpers::test_app;
#[tokio::test]
async fn logout_return_auth_ui() { 
    let app = test_app().await;
    let response = app.post_logout().await;
    println!("logout:{}", response.status());
    assert_eq!(response.status().as_u16(), 200);
}