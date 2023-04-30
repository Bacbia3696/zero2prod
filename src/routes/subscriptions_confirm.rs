use actix_web::{get, web, HttpResponse};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm pending subscription")]
#[get("/subscriptions/confirm")]
async fn subscribe_confirm(paramater: web::Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
