use actix_web::{post, web::Form, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct FormData {
    name: String,
    email: String,
}

#[post("/subscriptions")]
pub async fn subscribe(Form(form): Form<FormData>) -> impl Responder {
    HttpResponse::Ok().finish()
}
