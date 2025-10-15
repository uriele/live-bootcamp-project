use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }

        let email_regex = fancy_regex::Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
        let password_regex = fancy_regex::Regex::new(r"^(?!.*\s)(?=.*[0-9])(?=.*[!@#$%^&*])(?=.{8,})").unwrap();


        if !email_regex.is_match(&email.as_str()).unwrap() {
            return Err(UserStoreError::InvalidCredentials);
        }

        //password at least 8 letters and a number and a symbol, no spaces
        if !password_regex.is_match(&password.as_str()).unwrap() {
            return Err(UserStoreError::InvalidCredentials);
        }

            self.users.insert(user.email.clone(), user);
            Ok(())
        }

    pub fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }

    pub fn validate_credentials(&self, email: &str, password: &str) -> Result<bool, UserStoreError> {
        match self.get_user(&email) {
            Ok(user) => {
                if user.password == password {
                    Ok(true)
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }
            }
            Err(e) => Err(e),
        }
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    fn test_add_user() {
        let hash: HashmapUserStore = HashMapUserStore::new();
    }

    #[tokio::test]
    fn test_get_user() {
        todo!()
    }

    #[tokio::test]
    fn test_validate_user() {
        todo!()
    }
}