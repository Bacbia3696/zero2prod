use actix_web::{
    post,
    web::{Data, Form},
    HttpResponse, Responder,
};
use serde::Deserialize;
use sqlx::{query, types::chrono::Utc, PgPool};

#[derive(Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

#[post("/subscriptions")]
pub async fn subscribe(form: Form<FormData>, pool: Data<PgPool>) -> impl Responder {
    let _saved = query!(
        r#"INSERT INTO subscriptions(id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)"#,
        uuid::Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(pool.as_ref())
    .await
    .expect("Failed to insert DB");
    HttpResponse::Ok().finish()
}
