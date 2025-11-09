use std::error::Error;

use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    Email, Password, User,
};
use sqlx::PgPool;

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let email = user.email.as_ref();

        let existing_user = sqlx::query!("SELECT email FROM users WHERE email = $1", email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                println!("Database error: {}", e);
                UserStoreError::UnexpectedError
            })?;

        if existing_user.is_some() {
            return Err(UserStoreError::UserAlreadyExists);
        }

        let password = user.password.as_ref().to_string();
        let password_hash = match compute_password_hash(password).await {
            Ok(hash) => hash,
            Err(e) => {
                println!("Password hashing error: {}", e);
                return Err(UserStoreError::UnexpectedError);
            }
        };

        sqlx::query!(
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)",
            email,
            &password_hash,
            user.requires_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            println!("Database error: {}", e);
            UserStoreError::UnexpectedError
        })?;

        Ok(())
    }

    async fn validate_credentials(
        &self,
        email: Email,
        password: Password,
    ) -> Result<bool, UserStoreError> {
        let user = self.get_user(email).await?;

        verify_password_hash(
            user.password.as_ref().to_owned(),
            password.as_ref().to_owned(),
        )
        .await
        .map_err(|_| UserStoreError::InvalidCredentials)?;

        Ok(true)
    }

    async fn get_user(&self, email: Email) -> Result<User, UserStoreError> {
        sqlx::query!(
            r#"
            SELECT email, password_hash, requires_2fa
            FROM users
            WHERE email = $1
            "#,
            email.as_ref()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?
        .map(|row| {
            Ok(User {
                email: Email::parse(row.email).map_err(|_| UserStoreError::UnexpectedError)?,
                password: Password::hashed(row.password_hash)
                    .map_err(|_| UserStoreError::UnexpectedError)?,
                requires_2fa: row.requires_2fa,
            })
        })
        .ok_or(UserStoreError::UserNotFound)?
    }
}

async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let result = tokio::task::spawn_blocking(move || {
        let expected_password_hash: PasswordHash<'_> = PasswordHash::new(&expected_password_hash)?;

        Argon2::default()
            .verify_password(password_candidate.as_bytes(), &expected_password_hash)
            .map_err(|e| e.into())
    })
    .await;

    match result {
        Ok(res) => res,
        Err(e) => return Err(Box::new(e)),
    }
}

async fn compute_password_hash(password: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let result = tokio::task::spawn_blocking(move || {
        let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None)?,
        )
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

        Ok(password_hash)
    })
    .await;

    match result {
        Ok(res) => res,
        Err(e) => return Err(Box::new(e)),
    }
}
