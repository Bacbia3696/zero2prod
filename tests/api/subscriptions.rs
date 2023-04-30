use sqlx::query;
use tracing::info;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helper::spawn_app;

#[tokio::test]
async fn subscribe_persist_new_subscriber() {
    info!("subscribe_return_200_for_valid_form_data");
    let app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/v3/mail/send"))
        .and(method(http::method::Method::POST))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    let res = app.post_subscriptions(body).await;

    dbg!(&res);

    // make sure that subscriptions is saved
    let saved = query!("select email, name, status from subscriptions")
        .fetch_one(&app.pool)
        .await
        .expect("Failed to query DB");

    assert_eq!("ursula_le_guin@gmail.com", saved.email);
    assert_eq!("le guin", saved.name);
    assert_eq!("pending_confirmation", saved.status);
    assert_eq!(http::StatusCode::OK, res.status())
}

#[tokio::test]
async fn subscribe_return_400_when_data_is_missing() {
    info!("subscribe_return_400_when_data_is_missing");
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, err_message) in test_cases {
        let res = app.post_subscriptions(body).await;

        assert_eq!(
            http::StatusCode::BAD_REQUEST,
            res.status(),
            "The API did not failed when the payload was {}",
            err_message
        )
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        // Act
        let res = app.post_subscriptions(body).await;
        // Assert
        assert_eq!(
            400,
            res.status().as_u16(),
            "The API did not return a 200 OK when the payload was {}.",
            description
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/v3/mail/send"))
        .and(method(http::method::Method::POST))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    let res = app.post_subscriptions(body).await;
    assert_eq!(res.status(), http::StatusCode::OK)
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
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
    // Assert
    // Get the first intercepted request
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    // Parse the body as JSON, starting from raw bytes
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
    // Extract the link from one of the request fields.
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };
    let text_link = get_link(body["content"][0]["value"].as_str().unwrap());
    let html_link = get_link(body["content"][1]["value"].as_str().unwrap()); // The two links should be identical
    assert_eq!(text_link, html_link);
}
