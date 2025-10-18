
use super::{User,Email,Password};


#[async_trait::async_trait]
pub trait UserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn validate_credentials(&self, email: Email, password: Password) -> Result<bool, UserStoreError>;
    async fn get_user(&self, email: Email) -> Result<User, UserStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}
