#[derive(Debug)]
pub enum BannedTokenStoreError {
    UnexpectedError,
}

#[async_trait::async_trait]
pub trait BannedTokenStore {
    async fn ban_token(&mut self, token: String) -> Result<(), BannedTokenStoreError>;
    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
}
