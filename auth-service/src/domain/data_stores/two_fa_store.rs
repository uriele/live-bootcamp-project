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

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_two_fa_code_parse_valid() {
        let code_str = "123456".to_string();
        let code = TwoFACode::parse(code_str.clone());
        assert_eq!(code, Ok(TwoFACode(code_str)));
    }
    #[test]
    fn test_two_fa_code_parse_invalid() {
        let code_str = "12345a".to_string();
        let code = TwoFACode::parse(code_str.clone());
        assert_eq!(code, Err("Invalid 2FA code".into()));
    }   

    #[test]
    fn test_two_fa_code_default() {
        let code = TwoFACode::default();
        assert_eq!(code.as_ref().len(), 6);
        assert!(TwoFACode::parse(code.as_ref().to_string()).is_ok());
    }

    #[test]
    fn test_login_attempt_id_parse_valid() {
        let id_str = uuid::Uuid::new_v4().to_string();
        let id = LoginAttemptId::parse(id_str.clone());
        assert_eq!(id, Ok(LoginAttemptId(id_str)));
    }
    #[test]
    fn test_login_attempt_id_parse_invalid() {
        let id_str = "invalid-uuid".to_string();
        let id = LoginAttemptId::parse(id_str.clone());
        assert_eq!(id, Err("Invalid UUID".into()));
    }

    #[test]
    fn test_login_attempt_id_default() {
        let id = LoginAttemptId::default();
        assert!(uuid::Uuid::parse_str(id.as_ref()).is_ok());
    }


}