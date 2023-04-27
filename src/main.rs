use std::net::TcpListener;

use sqlx::PgPool;
use tracing_subscriber::EnvFilter;
use zero2prod::{configuration::get_configuration, run};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    telemetry();

    let configuration = get_configuration()?;

    let pool = PgPool::connect(&configuration.database.connection_string()).await?;

    eprintln!("start listening...");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))?;
    run(listener, pool).await?.await?;
    Ok(())
}

fn telemetry() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();
}
