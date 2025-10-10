use auth_service::Application;
use paste::paste;
use tokio::sync::OnceCell;
use uuid::Uuid;
macro_rules! post_test_functions {
    ($($name:ident),+ $(,)?) => {
        paste! {
            $(
                pub async fn [<post_ $name>](&self) -> reqwest::Response {
                    self.http_client
                        .post(&format!("{}/{}", &self.address, stringify!($name)).replace("_", "-"))
                        .send()
                        .await
                        .expect("Failed to execute request")
                }
            )*
        }
    }
}


static APP: OnceCell<TestApp> = OnceCell::const_new();
pub async fn test_app() -> &'static TestApp {
    APP.get_or_init(|| async {
        // Build and start your server once
        TestApp::new().await
    }).await
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp{
    pub async fn new() -> Self {
        let app= Application::build("127.0.0.1:0")
            .await
            .expect("Failed to build application");

        let address=format!("http://{}", app.address.clone());
        // clippy::let_underscore_future does 
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::new();

        return Self { address, http_client };
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
        // pass a reference to the string instead of allocating the string in memory
        .get(&format!("{}/", &self.address))
        .send()
        .await
        .expect("Failed to execute request")
    }



    post_test_functions!(login, logout, verify_2fa, verify_token);


    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

}
