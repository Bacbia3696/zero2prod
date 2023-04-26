use std::net::TcpListener;

use actix_web::{dev::Server, get, post, web::Form, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[derive(Deserialize, Debug)]
struct FormData {
    name: String,
    email: String,
}

#[post("/subscriptions")]
async fn subscribe(Form(form): Form<FormData>) -> impl Responder {
    dbg!(form);
    HttpResponse::Ok().finish()
}

pub async fn run(listener: TcpListener) -> eyre::Result<Server> {
    Ok(
        HttpServer::new(|| App::new().service(health_check).service(subscribe))
            .listen(listener)?
            .run(),
    )
}

#[cfg(test)]
mod tests;
