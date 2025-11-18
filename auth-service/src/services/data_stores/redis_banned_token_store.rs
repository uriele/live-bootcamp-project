use std::sync::Arc;
use tokio::sync::RwLock;
// Commands is a trait that provides high-level methods for Redis commands
use redis::{Commands, Connection};

use crate::{
    domain::data_stores::{
        BannedTokenStore,
        BannedTokenStoreError,
    },
    utils::auth::TOKEN_TTL_SECONDS
};

pub struct RedisBannedTokenStore {
    connection: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(connection: Arc<RwLock<Connection>>) -> Self {
        Self { connection }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn ban_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        let mut conn = self.connection.write().await;
        
        let banned_token= get_key(&token); 
        let _: () = conn.set_ex(banned_token, true, TOKEN_TTL_SECONDS as u64)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)?;
        Ok(())
    }

    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let mut conn = self.connection.write().await;
        let banned_token = get_key(token); // to avoid key collisions
        let exists: bool = conn.exists(banned_token)
            .map_err(|_| BannedTokenStoreError::UnexpectedError)?;
        Ok(exists)
    }
}

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";
fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}