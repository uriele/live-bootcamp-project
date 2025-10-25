use serde::{Serialize,Deserialize};
use rand::prelude::*;
use crate::domain::Email;

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



#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        uuid::Uuid::parse_str(&id)
            .map(|_| LoginAttemptId(id))
            .map_err(|_| "Invalid UUID".into())
    }
}


impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(uuid::Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TwoFACode(String);


impl TwoFACode {
    pub fn parse(code: String) -> Result<Self, String> {
        // Ensure `code` is a valid 6-digit code
        let reg_code= match fancy_regex::Regex::new(r"^\d{6}$"){
            Ok(reg) => reg,
            Err(_) => return Err("Failed to compile regex".into()),
        };

        match reg_code.is_match(&code) {
            Ok(true) =>  Ok(TwoFACode(code)), 
            _ => Err("Invalid 2FA code".into())
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
        TwoFACode(code)
    }
}

// TODO: Implement AsRef<str> for TwoFACode