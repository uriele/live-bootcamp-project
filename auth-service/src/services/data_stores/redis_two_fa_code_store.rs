use std::sync::Arc;
use redis::{Commands,Connection};

use secrecy::{Secret,ExposeSecret};
use serde::{Deserialize,Serialize};

use tokio::sync::RwLock;
use color_eyre::eyre::Context;
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
    #[tracing::instrument(name = "Add 2FA code in Redis", skip_all)]
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
        let val= TwoFATuple(login_attempt_id.as_ref().clone(), code.as_ref().clone());
        // 3. Serialize the TwoFATuple to a JSON string.
        let serialized_val= serde_json::to_string(&val)
            .wrap_err("Failed to serialize 2FA tuple")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;
        
        // 4. Call the set_ex command to set a new key/value pair with an expiration time (TTL).
        // The value should be he serialized 2FA tuple.
        // The expiration time should be set to TEN_MINUES_IN_SECONDS.
        // Return TwoFACodeStoreError::UnexpectedError if casting fails or the call to set_ex fails.
        let _: () = conn.set_ex(key, serialized_val, TEN_MINUTES_IN_SECONDS)
            .wrap_err("failed to set 2FA code in Redis")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;
        
        Ok(())
    }

    #[tracing::instrument(name = "Remove 2FA code in Redis", skip_all)]
    async fn remove_code(&mut self, email:&Email) -> Result<(),TwoFACodeStoreError> {
        let mut conn = self.connection.write().await;
        
        let key= get_key(email);

        let _: ()= conn.del(key)
            .wrap_err("failed to delete 2FA code in Redis")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(name = "Get 2FA code from Redis", skip_all)]
    async fn get_code(
        &self,
        email: &Email) -> Result<(LoginAttemptId,TwoFACode),TwoFACodeStoreError>{
            let mut conn= self.connection.write().await;
            let key= get_key(email);

            if !(conn.exists(key.clone())
                .wrap_err("failed to check existence of 2FA code in Redis")
                .map_err(TwoFACodeStoreError::UnexpectedError)?) {
                return Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
            };

            match conn.get::<_, String>(&key) {
                Ok(value) => {
                    let data: TwoFATuple = serde_json::from_str(&value)
                        .wrap_err("failed to deserialize 2FA tuple from Redis")
                        .map_err(TwoFACodeStoreError::UnexpectedError)?;

                    let login_attempt_id = LoginAttemptId::parse(data.0)
                        .map_err(TwoFACodeStoreError::UnexpectedError)?;
                    let two_fa_code = TwoFACode::parse(data.1)
                        .map_err(TwoFACodeStoreError::UnexpectedError)?;
                Ok((login_attempt_id,two_fa_code))
                },
                Err(_) => Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
            }
        }

} 



#[derive(Deserialize)]
struct TwoFATuple(pub Secret<String>,pub Secret<String>);

impl Serialize for TwoFATuple {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let tuple = (self.0.expose_secret(), self.1.expose_secret());
        tuple.serialize(serializer)
    }
}   

const TEN_MINUTES_IN_SECONDS: u64= 600;
const TWO_FA_CODE_PREFIX: &str= "two_fa_code:";
fn get_key(email: &Email) -> String {
    format!("{}{}",TWO_FA_CODE_PREFIX,email.as_ref().expose_secret())
}