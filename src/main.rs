use std::{net::TcpListener, time::Duration};

use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration::{get_configuration, AppSettings},
    run, telemetry, email_client::EmailClient,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    telemetry("info");

    let configuration = get_configuration()?;
    dbg!(&configuration);

    let AppSettings { host, port } = configuration.application;
    // Build an `EmailClient` using `configuration`
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let email_client = EmailClient::new(configuration.email_client.base_url, sender_email);

    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(configuration.database.withdb());

    let url = format!("{host}:{port}");
    eprintln!("start listening on {url}...");
    let listener = TcpListener::bind(url)?;
    run(listener, pool, email_client).await?.await?;
    Ok(())
}
