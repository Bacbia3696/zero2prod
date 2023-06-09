use crate::helper::spawn_app;

#[tokio::test]
async fn health_check_work() {
    // setup server
    let app = spawn_app().await;

    let res = reqwest::get(format!("{}/health_check", app.address))
        .await
        .expect("Failed to send request");

    assert_eq!(http::StatusCode::OK, res.status())
}
