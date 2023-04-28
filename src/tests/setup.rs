use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{migrate, Connection, Executor, PgConnection, PgPool};

use uuid::Uuid;

use crate::{
    configuration::{self, get_configuration},
    run, telemetry,
};

pub struct AppTest {
    pub address: String,
    pub pool: PgPool,
}

pub async fn spawn_app() -> AppTest {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();

    let mut configuration = get_configuration().unwrap();
    configuration.database.database_name = Uuid::new_v4().to_string();

    let pool = configure_database(&configuration.database).await;
    let server = run(listener.try_clone().unwrap(), PgPool::clone(&pool))
        .await
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);
    AppTest {
        address: format!("http://{}", listener.local_addr().unwrap().to_string()),
        pool,
    }
}

static TRACING: Lazy<()> = Lazy::new(|| telemetry("info"));

async fn configure_database(db: &configuration::DatabaseSettings) -> PgPool {
    Lazy::force(&TRACING);
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
