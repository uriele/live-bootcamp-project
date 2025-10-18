use std::collections::HashMap;
use crate::domain::*;

#[derive(Default, Clone)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}


impl HashmapUserStore {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            return Err(UserStoreError::UserAlreadyExists);
        }
        let email_regex = fancy_regex::Regex::new(r"^[\w\.-]+@[\w\.-]+\.\w+$").unwrap();
        let password_regex = fancy_regex::Regex::new(r"^(?!.*\s)(?=.*[0-9])(?=.*[!@#$%^&*])(?=.{8,})").unwrap();


        if !email_regex.is_match(&user.email.as_ref()).unwrap() {
            return Err(UserStoreError::InvalidCredentials);
        }

        //password at least 8 letters and a number and a symbol, no spaces
        if !password_regex.is_match(&user.password.as_ref()).unwrap() {
            return Err(UserStoreError::InvalidCredentials);
        }

            self.users.insert(user.email.clone(), user);
            Ok(())
        }

    async fn validate_credentials(&self, email: Email, password: Password) -> Result<bool, UserStoreError> {

        match self.get_user(email).await {
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

    async fn get_user(&self, email: Email) -> Result<User, UserStoreError> {
        self.users.get(&email).cloned().ok_or(UserStoreError::UserNotFound)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut hash: HashmapUserStore = HashmapUserStore::new();
        let valid_user = Email::parse("test_user@example.com".to_string()).unwrap();
        let valid_password = Password::parse("Password123!".to_string()).unwrap();
        let user = User::new_without_2fa(
            valid_user.clone(),
            valid_password.clone()
        );
        let existing_user = user.clone();

        let result_new= hash.add_user(user).await;
        assert_eq!(result_new, Ok(()));

        let result_duplicate = hash.add_user(existing_user).await;
        assert_eq!(result_duplicate, Err(UserStoreError::UserAlreadyExists));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut hash = HashmapUserStore::new();
        let valid_user = Email::parse("test_user@example.com".to_string()).unwrap();
        let valid_password = Password::parse("Password123!".to_string()).unwrap();
        let user = User::new_without_2fa(
            valid_user.clone(),
            valid_password.clone()
        );

        hash.add_user(user.clone()).await.unwrap();
        let retrieved_user = hash.get_user(valid_user.clone()).await;
        assert_eq!(retrieved_user, Ok(user));
        let non_existent_user = hash.get_user(Email::parse("non_existent_user@example.com".to_string()).unwrap()).await;
        assert_eq!(non_existent_user, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut hash = HashmapUserStore::new();
        let valid_user=Email::parse("test_user@example.com".to_string()).unwrap();
        let valid_password=Password::parse("Password123!".to_string()).unwrap();
        let invalid_user=Email::parse("invalid_user@example.com".to_string()).unwrap();
        let invalid_password=Password::parse("Password321!".to_string()).unwrap();    
        let user = User::new_without_2fa(
            valid_user.clone(),
            valid_password.clone()
        );

        hash.add_user(user.clone()).await.unwrap();

        let valid_credentials = hash.validate_credentials(valid_user.clone(), valid_password.clone()).await;
        assert_eq!(valid_credentials, Ok(true));

        let invalid_credentials = hash.validate_credentials(valid_user.clone(),invalid_password.clone()).await;
        assert_eq!(invalid_credentials, Err(UserStoreError::InvalidCredentials));

        let non_existent_user = hash.validate_credentials(
            invalid_user.clone(),
            valid_password.clone()
        ).await;
        assert_eq!(non_existent_user, Err(UserStoreError::UserNotFound));
    }
}