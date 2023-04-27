use std::net::TcpListener;

use actix_web::{dev::Server, web::Data, App, HttpServer};
use routes::{health_check, subscribe};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub mod configuration;
mod routes;
mod telemetry;
pub use telemetry::*;

pub async fn run(listener: TcpListener, pool: PgPool) -> eyre::Result<Server> {
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(subscribe)
            .app_data(Data::new(pool.clone()))
    })
    .listen(listener)?
    .run())
}

#[cfg(test)]
mod tests;
