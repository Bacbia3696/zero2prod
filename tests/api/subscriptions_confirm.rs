use reqwest::Url;
use sqlx::query;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helper::spawn_app;

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    let app = spawn_app().await;
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();
    assert_eq!(400, response.status())
}

#[tokio::test]
async fn confirmations_return_link_that_able_to_call() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/v3/mail/send"))
        .and(method(http::method::Method::POST))
        .respond_with(ResponseTemplate::new(200))
        // We are not setting an expectation here anymore // The test is focused on another aspect of the app // behaviour.
        .mount(&app.email_server)
        .await;
    // Act
    app.post_subscriptions(body).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let raw_confirmation_link = app.get_confirmation_link(email_request);
    let mut confirmation_link = Url::parse(&raw_confirmation_link).unwrap();
    confirmation_link.set_port(Some(app.port)).unwrap();
    // Let's make sure we don't call random APIs on the web
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
    // Act
    let response = reqwest::get(confirmation_link).await.unwrap();
    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn confirmations_ok_when_user_click_to_the_link() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/v3/mail/send"))
        .and(method(http::method::Method::POST))
        .respond_with(ResponseTemplate::new(200))
        // We are not setting an expectation here anymore // The test is focused on another aspect of the app // behaviour.
        .mount(&app.email_server)
        .await;
    // Act
    app.post_subscriptions(body).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let raw_confirmation_link = app.get_confirmation_link(email_request);
    let mut confirmation_link = Url::parse(&raw_confirmation_link).unwrap();
    confirmation_link.set_port(Some(app.port)).unwrap();
    reqwest::get(confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let saved = query!("select email,name,status from subscriptions")
        .fetch_one(&app.pool)
        .await
        .unwrap();

    assert_eq!("ursula_le_guin@gmail.com", saved.email);
    assert_eq!("le guin", saved.name);
    assert_eq!("confirmed", saved.status);
}
