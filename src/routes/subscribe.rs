use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use sqlx::{types::chrono::Utc, PgPool};
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};

#[derive(Deserialize, Debug)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        Ok(Self {
            email: SubscriberEmail::parse(value.email)?,
            name: SubscriberName::parse(value.name)?,
        })
    }
}

#[tracing::instrument(name = "Adding a new subscriber", skip(form, pool, email_client))]
#[post("/subscriptions")]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(new_subscriber) => new_subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let Ok(res) = insert_subscriber(&pool, &new_subscriber).await else {
        return HttpResponse::InternalServerError().finish();
    };
    let confirmation_link = "https://my-api.com/subscriptions/confirm";

    let res= email_client
        .send_email(
            SubscriberEmail::parse("bacbia@gmail.com".to_string()).unwrap(),
            "Welcome!",
            &format!(
                "Welcome to our newsletter!<br />\
                Click <a href=\"{}\">here</a> to confirm your subscription.",
                confirmation_link
            ),
            &format!(
                "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
                confirmation_link
            ),
        )
        .await
        .unwrap();
    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, $5)
"#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now(),
        "pending_confirmation",
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
