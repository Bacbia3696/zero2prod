use std::{net::TcpListener, time::Duration};

use sqlx::{postgres::PgPoolOptions, PgPool};
use zero2prod::{
    configuration::{get_configuration, AppSettings},
    run, telemetry,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    telemetry();

    let configuration = get_configuration()?;
    let AppSettings { host, port } = configuration.application;

    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy(&configuration.database.connection_string())?;

    let url = format!("{host}:{port}");
    eprintln!("start listening on {url}...");
    let listener = TcpListener::bind(url)?;
    run(listener, pool).await?.await?;
    Ok(())
}
