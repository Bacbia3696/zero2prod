use std::{net::TcpListener, os::unix::thread};

use sqlx::{migrate, query, Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use crate::{
    configuration::{self, get_configuration},
    run,
};

pub struct AppTest {
    pub address: String,
    pub pool: PgPool,
    configuration: configuration::Settings,
}

pub async fn spawn_test() -> AppTest {
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
        configuration,
    }
}

async fn configure_database(db: &configuration::DatabaseSettings) -> PgPool {
    // create DB
    PgConnection::connect(&db.connection_string_without_db())
        .await
        .expect("Failed to connect DB")
        .execute(format!(r#"create database "{}";"#, &db.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // connect to DB and run migration
    let pool = PgPool::connect(&db.connection_string())
        .await
        .expect("Failed to connect DB");
    migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migration");

    pool
}
