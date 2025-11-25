use crate::domain::Email;
use std::hash::{Hash};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use color_eyre::eyre::{eyre,Context,Result,Report};
use thiserror::Error;
use secrecy::{Secret,ExposeSecret};
#[async_trait::async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, Error)]
pub enum TwoFACodeStoreError {
    #[error("Login Attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for TwoFACodeStoreError{    
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}


#[derive(Debug, Clone, Deserialize)]
pub struct LoginAttemptId(Secret<String>);

impl PartialEq for LoginAttemptId {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Eq for LoginAttemptId {}

impl Hash for LoginAttemptId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

impl Serialize for LoginAttemptId {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.expose_secret())
    }
}



impl LoginAttemptId {
    pub fn parse(id: Secret<String>) -> Result<Self> {
        let _ =uuid::Uuid::parse_str(id.expose_secret())
            .wrap_err("Invalid login attempt id")?;
        Ok(LoginAttemptId(id.into()))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(Secret::new(uuid::Uuid::new_v4().to_string()))
    }
}

impl AsRef<Secret<String>> for LoginAttemptId {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TwoFACode(Secret<String>);

impl PartialEq for TwoFACode {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Hash for TwoFACode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}


impl Serialize for TwoFACode {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.expose_secret())
    }
}


impl Eq for TwoFACode {}


impl TwoFACode {
    pub fn parse(code: Secret<String>) -> Result<Self> {
        // Ensure `code` is a valid 6-digit code
        let reg_code = match fancy_regex::Regex::new(r"^\d{6}$") {
            Ok(reg) => reg,
            Err(_) => return Err(eyre!("Failed to compile regex")),
        };

        match reg_code.is_match(code.expose_secret()) {
            Ok(true) => Ok(Self(code)) ,
            _ => Err(eyre!("Invalid 2FA code")),
        }
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let code: String = (0..6)
            .map(|_| rng.gen_range(0..10))
            .map(|d| std::char::from_digit(d as u32, 10).unwrap())
            .collect();
        TwoFACode(Secret::new(code))
    }
}

impl AsRef<Secret<String>> for TwoFACode {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn is_ok<T, E>(res: &Result<T, E>) -> bool {
        matches!(res, Ok(_))
    }
    
    fn is_err<T, E>(res: &Result<T, E>) -> bool {
        matches!(res, Err(_))
    }


    #[test]
    fn test_two_fa_code_parse_valid() {
        let code_str = "123456".to_string();
        let code = TwoFACode::parse(code_str.clone().into());

        assert!(is_ok(&code));
        assert_eq!(code.unwrap(), TwoFACode(Secret::new(code_str)));
    }
    #[test]
    fn test_two_fa_code_parse_invalid() {
        let code_str = Secret::new("12345a".to_string());
        let code = TwoFACode::parse(code_str.clone());
        assert!(is_err(&code));
    }

    #[test]
    fn test_two_fa_code_default() {
        let code = TwoFACode::default();
        assert_eq!(code.as_ref().expose_secret().len(), 6);
        assert!(TwoFACode::parse(code.as_ref().clone()).is_ok());
    }

    #[test]
    fn test_login_attempt_id_parse_valid() {
        let id_str = Secret::new(uuid::Uuid::new_v4().to_string());
        let id = LoginAttemptId::parse(id_str.clone());
        assert!(is_ok(&id));
        assert_eq!(id.unwrap(), LoginAttemptId(id_str));
    }
    #[test]
    fn test_login_attempt_id_parse_invalid() {
        let id_str = Secret::new("invalid-uuid".to_string());
        let id = LoginAttemptId::parse(id_str.clone());
        assert!(is_err(&id));
    }

    #[test]
    fn test_login_attempt_id_default() {
        let id = LoginAttemptId::default();
        assert!(uuid::Uuid::parse_str(id.as_ref().expose_secret()).is_ok());
    }
    
}
