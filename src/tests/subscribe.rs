use sqlx::query;
use tracing::info;

use crate::tests::setup::*;

#[tokio::test]
async fn subscribe_return_200_for_valid_form_data() {
    info!("subscribe_return_200_for_valid_form_data");
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let res = client
        .post(format!("{}/subscriptions", app.address))
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("Failed to send request");

    // make sure that subscriptions is saved
    let saved = query!("select email, name from subscriptions")
        .fetch_one(&app.pool)
        .await
        .expect("Failed to query DB");

    assert_eq!("ursula_le_guin@gmail.com", saved.email);
    assert_eq!("le guin", saved.name);
    assert_eq!(http::StatusCode::OK, res.status())
}

#[tokio::test]
async fn subscribe_return_400_when_data_is_missing() {
    info!("subscribe_return_400_when_data_is_missing");
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, err_message) in test_cases {
        let res = client
            .post(format!("{}/subscriptions", app.address))
            .body(body)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .await
            .expect("Failed to send request");

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
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 200 OK when the payload was {}.",
            description
        );
    }
}
