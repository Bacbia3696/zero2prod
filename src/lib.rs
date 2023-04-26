use std::net::TcpListener;

use actix_web::{dev::Server, web::Data, App, HttpServer};
use routes::{health_check, subscribe};
use sqlx::PgPool;

pub mod configuration;
mod routes;

pub async fn run(listener: TcpListener, pool: PgPool) -> eyre::Result<Server> {
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(health_check)
            .service(subscribe)
    })
    .listen(listener)?
    .run())
}

#[cfg(test)]
mod tests;
