use std::{net::TcpListener, time::Duration};

use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration::{get_configuration, AppSettings},
    run, telemetry,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    telemetry("info");

    let configuration = get_configuration()?;
    dbg!(&configuration);
    let AppSettings { host, port } = configuration.application;

    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(configuration.database.withdb());

    let url = format!("{host}:{port}");
    eprintln!("start listening on {url}...");
    let listener = TcpListener::bind(url)?;
    run(listener, pool).await?.await?;
    Ok(())
}
