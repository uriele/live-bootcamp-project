use std::sync::Arc;
use tokio::sync::RwLock;
use crate::domain::{UserStore,BannedTokenStore, User, Email,Password, UserStoreError, TwoFACodeStore};
pub type UserStoreType = Arc<RwLock<dyn UserStore + Send + Sync >>;
pub type BannedTokenStoreType = Arc<RwLock<dyn BannedTokenStore + Send + Sync>>;
pub type TwoFACodeStoreType = Arc<RwLock<dyn TwoFACodeStore + Send + Sync>>;
#[derive(Clone)]
pub struct AppState{
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,   
    pub two_fa_code_store:  TwoFACodeStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType, banned_token_store: BannedTokenStoreType, two_fa_code_store: TwoFACodeStoreType) -> Self {
        let user_store = user_store;
        let banned_token_store = banned_token_store;
        let two_fa_code_store = two_fa_code_store;
        Self { user_store, banned_token_store, two_fa_code_store }
    }
}

#[async_trait::async_trait]
impl UserStore for AppState {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        self.user_store.write().await.add_user(user).await
    }

    async fn validate_credentials(&self, email: Email, password: Password) -> Result<bool, UserStoreError> {
        self.user_store.read().await.validate_credentials(email, password).await
    }

    async fn get_user(&self, email: Email) -> Result<User, UserStoreError> {
        self.user_store.read().await.get_user(email).await
    }
}

// Option 2: Trait Objects (Dynamic Dispatch - More Flexible)
// Uncomment and use this if you need runtime polymorphism
/*
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub user_store: Arc<RwLock<dyn UserStore + Send + Sync>>,
}

impl AppState {
    pub fn new(user_store: impl UserStore + Send + Sync + 'static) -> Self {
        let user_store = Arc::new(RwLock::new(user_store));
        Self { user_store }
    }
}

#[async_trait::async_trait]
impl UserStore for AppState {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        self.user_store.write().await.add_user(user).await
    }

    async fn validate_credentials(&self, email: &str, password: &str) -> Result<bool, UserStoreError> {
        self.user_store.read().await.validate_credentials(email, password).await
    }

    async   fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        self.user_store.read().await.get_user(email).await
    }
}
*/
