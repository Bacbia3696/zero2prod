use std::net::TcpListener;

use actix_web::{dev::Server, web::Data, App, HttpServer};
use email_client::EmailClient;
use routes::{health_check, subscribe};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub mod configuration;
mod routes;
mod telemetry;
pub use telemetry::*;
mod domain;
pub mod email_client;

pub async fn run(listener: TcpListener, pool: PgPool, email_client: EmailClient) -> eyre::Result<Server> {
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(subscribe)
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(email_client.clone()))
    })
    .listen(listener)?
    .run())
}

#[cfg(test)]
mod tests;
