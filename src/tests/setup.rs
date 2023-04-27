use std::net::TcpListener;

use sqlx::PgPool;

use crate::{configuration::get_configuration, run};

pub struct AppTest {
    pub address: String,
    pub pool: PgPool,
}

pub async fn spawn_test() -> AppTest {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();

    let configuration = get_configuration().unwrap();
    let pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect DB");
    let server = run(listener.try_clone().unwrap(), PgPool::clone(&pool))
        .await
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);
    AppTest {
        address: format!("http://{}", listener.local_addr().unwrap().to_string()),
        pool,
    }
}
