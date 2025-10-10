use axum::{ response::{Html}, routing::get, routing::post, serve::Serve, Router};
use tower_http::services::ServeDir;
use tokio::net::TcpListener;
use std::error::Error;

pub use axum::response::IntoResponse;
pub use axum::http::StatusCode;

pub mod utils;
pub mod routes;
use routes::*;
pub struct Application {
    server: Serve<Router,Router>,
    // address is exposed as a public field
    pub address: String,
}


impl Application {
    pub async fn build(address: &str) -> Result<Self,Box<dyn Error>> {
        // Move the Router definition from `main.rs` to here.
        // Also, remove the `hello` route.
        // We don't need it at this point!
        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/hello", get(hello_handler))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token));

        let listener = TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        // Create a new Application instance and return it
        //todo!()
        Ok(Self { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}

pub async fn hello_handler() -> Html<&'static str> {
    // TODO: Update this to a custom message!
    Html("<h1>Hello, Marco!</h1>")
}
