use std::net::TcpListener;

use actix_web::{dev::Server, HttpServer, App, web::Data};
use sqlx::postgres::PgPoolOptions;

use crate::{configuration::Settings, email_client::EmailClient, routes};

use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub async fn build(configuration: Settings) -> eyre::Result<Server> {
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.withdb());
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool, email_client).await
}

pub async fn run(
    listener: TcpListener,
    pool: PgPool,
    email_client: EmailClient,
) -> eyre::Result<Server> {
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(routes::health_check)
            .service(routes::subscribe)
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(email_client.clone()))
    })
    .listen(listener)?
    .run())
}
