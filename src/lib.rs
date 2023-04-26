use std::net::TcpListener;

use actix_web::{dev::Server, App, HttpServer};
use routes::{health_check, subscribe};

pub mod configuration;
mod routes;

pub async fn run(listener: TcpListener) -> eyre::Result<Server> {
    Ok(
        HttpServer::new(|| App::new().service(health_check).service(subscribe))
            .listen(listener)?
            .run(),
    )
}

#[cfg(test)]
mod tests;
