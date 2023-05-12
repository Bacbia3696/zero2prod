use once_cell::sync::Lazy;
use sqlx::{migrate, Connection, Executor, PgConnection, PgPool};

use uuid::Uuid;

use wiremock::MockServer;
use zero2prod::{
    configuration::{self, get_configuration},
    startup::Application,
    telemetry,
};

static TRACING: Lazy<()> = Lazy::new(|| telemetry("zero2prod=info"));

pub struct AppTest {
    pub address: String,
    pub pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
}

impl AppTest {
    pub async fn post_subscriptions(&self, body: &str) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.to_string())
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_link(&self, email_request: &wiremock::Request) -> String {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            links[0].as_str().to_owned()
        };
        let text_link = get_link(body["content"][0]["value"].as_str().unwrap());
        let html_link = get_link(body["content"][1]["value"].as_str().unwrap()); // The two links should be identical
        assert_eq!(text_link, html_link);
        text_link
    }
}

// TODO: refactor to use setup module
pub async fn spawn_app() -> AppTest {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;
    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        // Use the mock server as email API
        c.email_client.base_url = email_server.uri();
        c
    };

    let pool = configure_database(&configuration.database).await;
    let app = Application::build(configuration).await.unwrap();

    let _ = tokio::spawn(app.server);
    AppTest {
        address: format!("http://localhost:{}", app.port),
        pool,
        email_server,
        port: app.port,
    }
}

async fn configure_database(db: &configuration::DatabaseSettings) -> PgPool {
    // create DB
    PgConnection::connect_with(&db.without_db())
        .await
        .expect("Failed to connect DB")
        .execute(format!(r#"create database "{}";"#, &db.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // connect to DB and run migration
    let pool = PgPool::connect_with(db.withdb())
        .await
        .expect("Failed to connect DB");
    migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migration");

    pool
}
