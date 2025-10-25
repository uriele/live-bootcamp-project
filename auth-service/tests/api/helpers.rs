use std::sync::Arc;

use reqwest::cookie::Jar;
use auth_service::Application;
//use paste::paste;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use auth_service::app_state::{BannedTokenStoreType,TwoFACodeStoreType,UserStoreType, EmailClientType};
use auth_service::services::{HashmapUserStore,HashsetBannedTokenStore,HashmapTwoFACodeStore};
use auth_service::utils::constants::test;
use auth_service::utils::auth::generate_auth_token;
use auth_service::domain::Email;
use std::ops::Deref;

use auth_service::services::MockEmailClient;
use auth_service::app_state::AppState;
// use tokio::sync::OnceCell;
use uuid::Uuid;

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub user_store: UserStoreType,
    pub email_client: EmailClientType,
    pub http_client: reqwest::Client,
}




impl TestApp{
    pub async fn new() -> Self {

        let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
        let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
        let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
        let email_client = Arc::new(RwLock::new(MockEmailClient));

        let app_state = AppState::new(user_store.clone(), banned_token_store.clone(), two_fa_code_store.clone(), email_client.clone());
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
            banned_token_store,
            two_fa_code_store,
            user_store,
            email_client,
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



    // post_test_functions!( verify_2fa);


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
#[derive(Debug, Clone,Serialize, Deserialize)]
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