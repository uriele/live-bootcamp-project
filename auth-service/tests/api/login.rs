use crate::helpers::test_app;

#[tokio::test]
async fn login_return_auth_ui() { 
    let app = test_app().await;
    let response = app.post_login().await;
    println!("login:{}", response.status());
    assert_eq!(response.status().as_u16(), 200);
}