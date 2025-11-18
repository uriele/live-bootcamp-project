use std::sync::Arc;
use redis::{Commands,Connection};

use serde::{Deserialize,Serialize};

use tokio::sync::RwLock;

use crate::domain::{data_stores::{
    LoginAttemptId,
    TwoFACode,TwoFACodeStore,
    TwoFACodeStoreError},
    Email
};


pub struct RedisTwoFACodeStore {
    connection: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore{
    pub fn new(connection: Arc<RwLock<Connection>>) -> Self {
        Self{connection}
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore{
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode
    ) -> Result<(),TwoFACodeStoreError> {
        let mut conn = self.connection.write().await;
        
        // 1. Create a new key using the get_key helper function.
        let key= get_key(&email);
        // 2. Create a TwoFATuple with login attempt id and the code
        let val= TwoFATuple(login_attempt_id.as_ref().to_string(),code.as_ref().to_string());
        // 3. Serialize the TwoFATuple to a JSON string.
        let serialized_val= match serde_json::to_string(&val)
        {
            Ok(s)=> s,
            Err(_)=> return Err(TwoFACodeStoreError::UnexpectedError),
        };
        // 4. Call the set_ex command to set a new key/value pair with an expiration time (TTL).
        // The value should be he serialized 2FA tuple.
        // The expiration time should be set to TEN_MINUES_IN_SECONDS.
        // Return TwoFACodeStoreError::UnexpectedError if casting fails or the call to set_ex fails.
        let _: () = conn.set_ex(key, serialized_val, TEN_MINUTES_IN_SECONDS)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
        Ok(())
    }

    async fn remove_code(&mut self, email:&Email) -> Result<(),TwoFACodeStoreError> {
        let mut conn = self.connection.write().await;
        
        let key= get_key(email);

        let _ = conn.del(key)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email) -> Result<(LoginAttemptId,TwoFACode),TwoFACodeStoreError>{
            let mut conn= self.connection.write().await;
            let key= get_key(email);

            if !(conn.exists(key.clone())
                .map_err(|_| TwoFACodeStoreError::UnexpectedError)?) {
                return Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
            };

            let serialized_val: String = conn.get(key)
                .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

            let deserialized_val: TwoFATuple = serde_json::from_str(&serialized_val)
                .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

            let login_attempt_id = LoginAttemptId::parse(deserialized_val.0)
                .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

            let two_fa_code= TwoFACode::parse(deserialized_val.1)
                .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

            Ok((login_attempt_id,two_fa_code))
        }
    

} 



#[derive(Serialize,Deserialize)]
struct TwoFATuple(pub String,pub String);

const TEN_MINUTES_IN_SECONDS: u64= 600;
const TWO_FA_CODE_PREFIX: &str= "two_fa_code:";
fn get_key(email: &Email) -> String {
    format!("{}{}",TWO_FA_CODE_PREFIX,email.as_ref())
}