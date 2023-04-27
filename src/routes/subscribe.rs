use actix_web::{
    post,
    web::{Data, Form},
    HttpResponse, Responder,
};
use serde::Deserialize;
use sqlx::{query, types::chrono::Utc, PgPool};
use tracing::{info, Instrument};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
name = "Adding a new subscriber", skip(form, pool),
fields(
    request_id = %Uuid::new_v4(),
    subscriber_email = %form.email,
    subscriber_name= %form.name
    )
)]
#[post("/subscriptions")]
pub async fn subscribe(pool: Data<PgPool>, form: Form<FormData>) -> impl Responder {
    info!("subscribe");
    let query_span = tracing::info_span!("Saving new subscriber details in the database");
    let res = query!(
        r#"INSERT INTO subscriptions(id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        uuid::Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(pool.as_ref())
    .instrument(query_span)
    .await;

    match res {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(error) => {
            tracing::error!(?error, "Failed save subscriber",);
            HttpResponse::InternalServerError().finish()
        }
    }
}
