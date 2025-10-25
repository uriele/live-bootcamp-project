use std::sync::Arc;

use reqwest::cookie::Jar;
use auth_service::Application;
use paste::paste;
use tokio::sync::RwLock;
use auth_service::services::HashmapUserStore;
use auth_service::utils::constants::test;
use auth_service::utils::auth::generate_auth_token;
use auth_service::domain::Email;
use std::ops::Deref;

// use tokio::sync::OnceCell;
use uuid::Uuid;
macro_rules! post_test_functions {
    ($($name:ident),+ $(,)?) => {
        paste! {
            $(
                pub async fn [<post_ $name>](&self) -> reqwest::Response {
                    self.http_client
                        .post(&format!("{}/{}", &self.address, stringify!($name)).replace("_", "-"))
                        .send()
                        .await
                        .expect("Failed to execute request")
                }
            )*
        }
    }
}

/*
static APP: OnceCell<TestApp> = OnceCell::const_new();
pub async fn test_app() -> &'static TestApp {
    APP.get_or_init(|| async {
        // Build and start your server once
        TestApp::new().await
    }).await
}
*/
pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
}




impl TestApp{
    pub async fn new() -> Self {

        let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    
    
        let app_state = auth_service::app_state::AppState::new(user_store);
        let app= Application::build(app_state,test::APP_ADDRESS)
            .await
            .expect("Failed to build application");

        let address=format!("http://{}", app.address.clone());
        // clippy::let_underscore_future does 
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        Self {
            address,
            cookie_jar,
            http_client,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
        // pass a reference to the string instead of allocating the string in memory
        .get(&format!("{}/", &self.address))
        .send()
        .await
        .expect("Failed to execute request")
    }



    post_test_functions!( verify_2fa);


    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    
    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

}


// A fake JWT struct for testing purposes
pub struct FakeJWT(String);

impl FakeJWT{
    pub fn parse(email:String) -> Self {
        Self(generate_auth_token(&Email::parse(email).unwrap()).unwrap())    
    }
}

impl Deref for FakeJWT {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}