
use crate::domain::{BannedTokenStore,BannedTokenStoreError};
use std::collections::HashSet;
use serde::{Serialize, Deserialize};
#[derive(Default, Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct HashsetBannedTokenStore {
    banned_tokens: HashSet<String>
}


#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn ban_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        self.banned_tokens.insert(token);
        Ok(())
    }

    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.banned_tokens.contains(token))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_add_token() {
        let mut store = HashsetBannedTokenStore::default();
        let token = "test_token".to_owned();

        let result = store.ban_token(token.clone()).await;

        assert!(result.is_ok());
        assert!(store.banned_tokens.contains(&token));
    }

    #[tokio::test]
    async fn test_contains_token() {
        let mut store = HashsetBannedTokenStore::default();
        let token = "test_token".to_owned();
        store.banned_tokens.insert(token.clone());

        let result = store.is_token_banned(&token).await;

        assert!(result.unwrap());
    }
}