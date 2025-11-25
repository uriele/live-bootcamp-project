use std::sync::Arc;
use tokio::sync::RwLock;
// Commands is a trait that provides high-level methods for Redis commands
use redis::{Commands, Connection};
use color_eyre::eyre::{Context};
use secrecy::{ExposeSecret,Secret};

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
    #[tracing::instrument(name = "Ban token in Redis", skip(self, token))]
    async fn ban_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError> {
        let mut conn = self.connection.write().await;
        

        let ttl: u64 = TOKEN_TTL_SECONDS
            .try_into()
            .wrap_err("failed to cast TOKEN_TTL_SECONDS to u64") // New!
            .map_err(BannedTokenStoreError::UnexpectedError)?; // Updated!



        let banned_token= get_key(token.expose_secret()); // to avoid key collisions
        let _: () = conn.set_ex(banned_token, true, ttl)
            .wrap_err("failed to set banned token in Redis") // New!
            .map_err(BannedTokenStoreError::UnexpectedError)?; // Updated!
        Ok(())
    }

    #[tracing::instrument(name = "Check if token is banned in Redis", skip(self, token))]
    async fn is_token_banned(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError> {
        let mut conn = self.connection.write().await;
        let banned_token = get_key(token.expose_secret()); // to avoid key collisions
        let exists: bool = conn.exists(banned_token)
            .wrap_err("failed to check if token exists in Redis") // New!
            .map_err(BannedTokenStoreError::UnexpectedError)?; // Updated!
        Ok(exists)
    }
}

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";
fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}