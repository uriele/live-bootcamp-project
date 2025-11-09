use auth_service::app_state::AppState;
use auth_service::services::{
    data_stores::postgres_user_store::PostgresUserStore, HashmapTwoFACodeStore,
    HashsetBannedTokenStore, MockEmailClient,
};
use auth_service::utils::{configure_postgres::configure_postgresql, constants::prod};
use auth_service::Application;
use std::sync::Arc;
use tokio::sync::RwLock;
#[tokio::main]
async fn main() {
    let pg_pool = configure_postgresql().await;

    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    //let user_store = Arc::new(RwLock::new(HashmapUserStore::default()));
    let banned_token_store = Arc::new(RwLock::new(HashsetBannedTokenStore::default()));
    let two_fa_code_store = Arc::new(RwLock::new(HashmapTwoFACodeStore::default()));
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
