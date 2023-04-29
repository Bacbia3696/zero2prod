use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{migrate, Connection, Executor, PgConnection, PgPool};

use uuid::Uuid;

use zero2prod::{
    configuration::{self, get_configuration},
    email_client::EmailClient,
    telemetry, startup::run,
};

static TRACING: Lazy<()> = Lazy::new(|| telemetry("info"));

pub struct AppTest {
    pub address: String,
    pub pool: PgPool,
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
}

// TODO: refactor to use setup module
pub async fn spawn_app() -> AppTest {
    Lazy::force(&TRACING);

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration."); // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        c
    };

    let pool = configure_database(&configuration.database).await;

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email.clone(),
        configuration.email_client.authorization_token,
        timeout,
    );
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address).unwrap();
    let server = run(
        listener.try_clone().unwrap(),
        PgPool::clone(&pool),
        email_client,
    )
    .await
    .expect("Failed to bind address");
    let _ = tokio::spawn(server);
    AppTest {
        address: format!("http://{}", listener.local_addr().unwrap().to_string()),
        pool,
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
