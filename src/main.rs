use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::{configuration::get_configuration, run, telemetry};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    telemetry();

    let configuration = get_configuration()?;
    let port = configuration.application_port;

    let pool = PgPool::connect(&configuration.database.connection_string()).await?;

    eprintln!("start listening on {port}...");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    run(listener, pool).await?.await?;
    Ok(())
}
