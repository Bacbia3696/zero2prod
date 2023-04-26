use std::net::TcpListener;

use crate::run;

#[tokio::test]
async fn health_check_work() {
    // setup server
    let address = spawn_test().await;

    let res = reqwest::get(format!("{address}/health_check"))
        .await
        .expect("Failed to send request");

    assert_eq!(http::StatusCode::OK, res.status())
}

#[tokio::test]
async fn subscribe_return_200_for_valid_form_data() {
    let address = spawn_test().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let res = client
        .post(format!("{address}/subscriptions"))
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(http::StatusCode::OK, res.status())
}

#[tokio::test]
async fn subscribe_return_400_when_data_is_missing() {
    let address = spawn_test().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, err_message) in test_cases {
        let res = client
            .post(format!("{address}/subscriptions"))
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

async fn spawn_test() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let server = run(listener.try_clone().unwrap())
        .await
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://{}", listener.local_addr().unwrap().to_string())
}
