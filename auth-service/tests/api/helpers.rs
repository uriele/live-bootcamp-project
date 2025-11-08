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
use auth_service::{
    services::data_stores::postgres_user_store::PostgresUserStore,
    get_postgres_pool
};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use auth_service::utils::constants::POSTGRES_URL;
use auth_service::services::MockEmailClient;
use auth_service::app_state::AppState;
// use tokio::sync::OnceCell;

use std::str::FromStr;
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
    pub db_name: String,
    pub clean_up_called: bool,
}




impl TestApp{
    pub async fn new() -> Self {

        let db_name = Uuid::new_v4().to_string();
        let pg_pool = configure_postgresql(&db_name).await;


        let clean_up_called= false;

        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
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
            db_name,
            clean_up_called
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


    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    
    pub async fn clean_up(&mut self) {
        if self.clean_up_called {
            return;
        }

        delete_database(&self.db_name).await;

        self.clean_up_called = true;
    }
}

impl Drop for TestApp{
        fn drop(&mut self) {
        if !self.clean_up_called {
            panic!("TestApp::clean_up was not called before dropping TestApp");
        }
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


async fn configure_postgresql(db_name: &str) -> PgPool {
    let postgresql_conn_url = POSTGRES_URL.to_owned();

    // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!")
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");


    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}


async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = POSTGRES_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}