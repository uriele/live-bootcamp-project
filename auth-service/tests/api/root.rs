use crate::helpers::TestApp;
#[tokio::test]
async fn root_return_auth_ui() { 
    let app = TestApp::new().await;
    let response = app.get_root().await;
    println!("root:{}", response.status());
    println!("{:#?}", response.headers());
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
}