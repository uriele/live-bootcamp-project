use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

pub const JWT_COOKIE_NAME: &str = "jwt";

lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
    pub static ref YOUR_IP: String= set_ip();
    //TODO: add new docker env as static ref
    pub static ref POSTGRES_PASSWORD: String=set_db_password();
    pub static ref POSTGRES_PORT: String=set_db_port();
    pub static ref POSTGRES_HOST: String=set_db_host();
    pub static ref POSTGRES_USER: String=set_db_user();
    pub static ref POSTGRES_URL: String=set_db_url();
}


fn set_db_password() -> String{
    dotenv().ok();
    match std_env::var(env::POSTGRES_PASSWORD_ENV_VAR) {
        Ok(val) => if !val.is_empty() {
            return val;
        }
        _ => ()
    }
    return String::from("password123")
}   

fn set_db_port() -> String{
    dotenv().ok();
    match std_env::var(env::POSTGRES_PORT_ENV_VAR) {
        Ok(val) => if !val.is_empty() {
            return val;
        }
        _ => ()
    }
    return String::from("5432")
}

fn set_db_host() -> String{
    dotenv().ok();
    match std_env::var(env::POSTGRES_HOST_ENV_VAR) {
        Ok(val) => if !val.is_empty() {
            return val;
        }
        _ => ()
    }
    return String::from("localhost")
}


fn set_db_user() -> String{
    dotenv().ok();
    match std_env::var(env::POSTGRES_USER_ENV_VAR) {
        Ok(val) => if !val.is_empty() {
            return val;
        }
        _ => ()
    }
    return String::from("postgres")
}


fn set_db_url() -> String{
    dotenv().ok();
    match std_env::var(env::DATABASE_URL_ENV_VAR) {
        Ok(val) => if !val.is_empty() {
            return val;
        }
        _ => ()
    }
    let db_url=format!("postgres://{}:{}@{}:{}",*POSTGRES_USER,*POSTGRES_PASSWORD,*POSTGRES_HOST,*POSTGRES_PORT);
    return db_url
}


fn set_ip() -> String{
    dotenv().ok();

    match std_env::var(env::YOUR_IP_ENV_VAR) {
        Ok(val) => if !val.is_empty() {
            return val;
        }
        _ => ()
    }

    return String::from("localhost")


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
    pub const YOUR_IP_ENV_VAR: &str = "YOUR_IP";
    pub const POSTGRES_DB_ENV_VAR: &str = "POSTGRES_DB";
    pub const POSTGRES_PASSWORD_ENV_VAR: &str = "POSTGRES_PASSWORD";
    pub const POSTGRES_PORT_ENV_VAR: &str = "POSTGRES_PORT";
    pub const POSTGRES_HOST_ENV_VAR: &str = "POSTGRES_HOST";
    pub const POSTGRES_USER_ENV_VAR: &str = "POSTGRES_USER";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";

}


pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}