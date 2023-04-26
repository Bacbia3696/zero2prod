use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::{configuration::get_configuration, run};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    println!("Listen http server...");

    let configuration = get_configuration()?;

    let pool = PgPool::connect(&configuration.database.connection_string()).await?;

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))?;
    run(listener, pool).await?.await?;
    Ok(())
}
