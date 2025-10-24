use axum::{ response::{Html}, routing::get, routing::post, serve::Serve, Router, Json};
use tower_http::services::ServeDir;
use tokio::net::TcpListener;
use std::error::Error;

pub use axum::response::IntoResponse;
pub use axum::http::StatusCode;

pub mod utils;
pub mod routes;
pub mod domain;
pub mod services;
pub mod app_state;
use routes::*;
use app_state::AppState;
use domain::AuthAPIErrors;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIErrors {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AuthAPIErrors::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIErrors::WrongEmailOrPassword => (StatusCode::UNAUTHORIZED, "Wrong email or password"),
            AuthAPIErrors::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIErrors::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            AuthAPIErrors::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error"),
        };

        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });

        (status, body).into_response()
    }
}   




pub struct Application {
    server: Serve<Router,Router>,
    // address is exposed as a public field
    pub address: String,
}


impl Application {
    pub async fn build<T>(app_state: AppState<T>, address: &str) -> Result<Self, Box<dyn Error>> 
    where T: domain::UserStore + Send + Sync + 'static + Clone
    {
        // Move the Router definition from `main.rs` to here.
        // Also, remove the `hello` route.
        // We don't need it at this point!
        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/hello", get(hello_handler))
            .route("/signup", post(signup::<T>))
            .route("/login", post(login::<T>))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token))
            .with_state(app_state);

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
