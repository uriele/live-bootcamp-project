use auth_service::app_state::AppState;
use auth_service::services::{
    data_stores::postgres_user_store::PostgresUserStore, 
    data_stores::{redis_banned_token_store::RedisBannedTokenStore,
    redis_two_fa_code_store::RedisTwoFACodeStore},
     MockEmailClient,//HashsetBannedTokenStore,HashmapTwoFACodeStore,
};
use auth_service::utils::{
    tracing::init_tracing,
    configure_postgres::configure_postgresql,
    configure_redis::configure_redis,
    constants::prod};
use auth_service::Application;
use std::sync::Arc;
use tokio::sync::RwLock;


#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre"); // New!
    init_tracing().expect("Failed to initialize tracing"); // Updated!
    let pg_pool = configure_postgresql().await;
    let redis_connection = Arc::new(RwLock::new(configure_redis().await));
    

    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    //let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_connection.clone())));
    //let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));

    let two_fa_code_store= Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_connection)));
    // let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
    let email_client = Arc::new(RwLock::new(MockEmailClient));
    let app_state = AppState::new(
        user_store,
        banned_token_store,
        two_fa_code_store,
        email_client,
    );

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}


