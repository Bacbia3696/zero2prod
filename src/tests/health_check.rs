use std::net::TcpListener;

use sqlx::{query, PgPool};

use crate::{configuration::get_configuration, run};

#[tokio::test]
async fn health_check_work() {
    // setup server
    let app = spawn_test().await;

    let res = reqwest::get(format!("{}/health_check", app.address))
        .await
        .expect("Failed to send request");

    assert_eq!(http::StatusCode::OK, res.status())
}

#[tokio::test]
async fn subscribe_return_200_for_valid_form_data() {
    let app = spawn_test().await;
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
    let app = spawn_test().await;
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

struct AppTest {
    address: String,
    pool: PgPool,
}

async fn spawn_test() -> AppTest {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();

    let configuration = get_configuration().unwrap();
    let pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect DB");
    let server = run(listener.try_clone().unwrap(), PgPool::clone(&pool))
        .await
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);
    AppTest {
        address: format!("http://{}", listener.local_addr().unwrap().to_string()),
        pool,
    }
}
