use crate::{get_redis_client, utils::constants::REDIS_HOST};

pub async fn configure_redis() -> redis::Connection {
    // Implementation for configuring Redis connection
    println!("Configuring Redis connection...");
    println!("Redis host name: {}", REDIS_HOST.to_string());
    get_redis_client(REDIS_HOST.to_owned())
        .await
        .expect("Failed to create Redis client")
        .get_connection()
        .expect("Failed to connect to Redis")
}