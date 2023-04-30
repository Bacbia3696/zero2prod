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
    let _app = spawn_app();
    let _body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
}
