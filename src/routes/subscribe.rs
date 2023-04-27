use actix_web::{
    post,
    web::{Data, Form},
    HttpResponse, Responder,
};
use serde::Deserialize;
use sqlx::{query, types::chrono::Utc, PgPool};
use tracing::Instrument;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

#[post("/subscriptions")]
pub async fn subscribe(pool: Data<PgPool>, form: Form<FormData>) -> impl Responder {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name= %form.name
    );
    let _request_span_guard = request_span.enter();
    // We do not call `.enter` on query_span!
    // `.instrument` takes care of it at the right moments // in the query future lifetime
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
        Ok(_) => {
            tracing::info!("New subscriber details have been saved",);
            HttpResponse::Ok().finish()
        }
        Err(error) => {
            tracing::error!(?error, "Failed save subscriber",);
            HttpResponse::InternalServerError().finish()
        }
    }
}
