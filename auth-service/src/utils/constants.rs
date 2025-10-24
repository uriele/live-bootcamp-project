use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

pub const JWT_COOKIE_NAME: &str = "jwt";

lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
}

fn set_token() -> String {
    dotenv().ok();
    
    let secret = std_env::var(env::JWT_SECRET_ENV_VAR)
        .expect("JWT_SECRET must be set");
    if secret.is_empty() {
        panic!("JWT_SECRET cannot be empty");
    }
    secret
}


pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
}
