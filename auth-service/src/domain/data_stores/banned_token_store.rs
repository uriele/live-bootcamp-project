
use thiserror::Error;
use color_eyre::eyre::{Report};
use secrecy::Secret;
#[derive(Debug,Error)]
pub enum BannedTokenStoreError {
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

#[async_trait::async_trait]
pub trait BannedTokenStore {
    async fn ban_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError>;
    async fn is_token_banned(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError>;
}
