use std::sync::Arc;
use secrecy::{ExposeSecret, Secret};
use auth_service::Application;
use reqwest::cookie::Jar;
//use paste::paste;
use auth_service::app_state::AppState;
use auth_service::app_state::{
    BannedTokenStoreType, EmailClientType, TwoFACodeStoreType, UserStoreType,
};
use auth_service::domain::Email;
use auth_service::services::MockEmailClient;
use auth_service::services::{HashmapTwoFACodeStore, HashmapUserStore, HashsetBannedTokenStore};
use auth_service::utils::auth::generate_auth_token;
use auth_service::utils::constants::test;
use auth_service::utils::constants::{POSTGRES_URL, REDIS_HOST};
use auth_service::{
    get_postgres_pool, get_redis_client, services::data_stores::{
        postgres_user_store::PostgresUserStore,
        redis_banned_token_store::RedisBannedTokenStore,
        redis_two_fa_code_store::RedisTwoFACodeStore,
    }
};
use serde::{Deserialize};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use std::ops::Deref;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
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
    server_handle: Option<JoinHandle<()>>,
}

impl TestApp {
    pub async fn new() -> Self {
        let db_name = Uuid::new_v4().to_string();
        let pg_pool = configure_postgresql(&db_name).await;
        let redis_connection = Arc::new(RwLock::new(configure_redis().await));

        let clean_up_called = false;

        let user_store: UserStoreType = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_token_store: BannedTokenStoreType =
            Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_connection.clone())));
            
            //Arc::new(RwLock::new(HashsetBannedTokenStore::default()));


        let two_fa_code_store: TwoFACodeStoreType =
            Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_connection)));
            //Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
        let email_client: EmailClientType = Arc::new(RwLock::new(MockEmailClient));

        let app_state = AppState::new(
            user_store.clone(),
            banned_token_store.clone(),
            two_fa_code_store.clone(),
            email_client.clone(),
        );
        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build application");

        let address = format!("http://{}", app.address.clone());
        let server_handle = tokio::spawn(async move {
            if let Err(err) = app.run().await {
                eprintln!("Test server exited with error: {err}");
            }
        });

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
            clean_up_called,
            server_handle: Some(server_handle),
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

        if let Some(handle) = self.server_handle.take() {
            handle.abort();
            let _ = handle.await;
        }

        let user_store: UserStoreType = Arc::new(RwLock::new(HashmapUserStore::default()));
        self.user_store = user_store;

        let banned_store: BannedTokenStoreType =
            Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
        self.banned_token_store = banned_store;

        let two_fa_store: TwoFACodeStoreType =
            Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
        self.two_fa_code_store = two_fa_store;

        let email_client: EmailClientType = Arc::new(RwLock::new(MockEmailClient));
        self.email_client = email_client;

        if let Err(err) = delete_database(&self.db_name).await {
            eprintln!("Failed to drop the database {}: {err}", self.db_name);
        }

        self.clean_up_called = true;
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.clean_up_called {
            if let Some(handle) = self.server_handle.take() {
                handle.abort();
            }
            eprintln!(
                "WARNING: TestApp::clean_up was not called before dropping TestApp for db {}",
                self.db_name
            );
        }
    }
}

// A fake JWT struct for testing purposes
#[derive(Debug, Clone,Deserialize)]
pub struct FakeJWT(Secret<String>);

impl FakeJWT {
    pub fn parse(email: Secret<String>) -> FakeJWT {
        FakeJWT(generate_auth_token(&Email::parse(email).unwrap()).unwrap())
    }
}

impl Deref for FakeJWT {
    type Target = Secret<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<Secret<String>> for FakeJWT {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

pub async fn configure_redis() -> redis::Connection {
    // Implementation for configuring Redis connection
    println!("Configuring Redis connection...");
    println!("Redis host name: {}", REDIS_HOST.to_string());
    get_redis_client(REDIS_HOST.to_owned())
        .await
        .expect("Failed to create Redis client")
        .get_connection()
        .expect("Failed to connect to Redis")
}


async fn configure_postgresql(db_name: &str) -> PgPool {
    let postgresql_conn_url = POSTGRES_URL.expose_secret().to_owned();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    println!("database_name: {}", postgresql_conn_url_with_db);
    // Create a new connection pool and return it
    get_postgres_pool(&Secret::new(postgresql_conn_url_with_db))
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

async fn delete_database(db_name: &str) -> Result<(), sqlx::Error> {
    let postgresql_conn_url: String = POSTGRES_URL.expose_secret().to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options).await?;

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
        .await?;

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE IF EXISTS "{}";"#, db_name).as_str())
        .await?;

    Ok(())
}
