use reqwest::Url;
use serde_json::json;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helper::{spawn_app, AppTest};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = json!({
            "title": "Newsletters title",
            "content": {
                "text": "plain text",
                "html": "<h1>Hello<h1>",
            }
    });

    let res = app.post_newsletters(&newsletter_request_body).await;
    assert_eq!(res.status(), http::StatusCode::OK);
    // Mock verifies on Drop that we haven't sent the newsletter email
}

async fn create_unconfirmed_subscriber(app: &AppTest) -> String {
    let body = "name=dat&email=bacbia%40gmai.com";
    let _mock_guard = Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body)
        .await
        .error_for_status()
        .unwrap();

    let email_request = app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_link(&email_request)
}

async fn create_confirmed_subscriber(app: &AppTest) {
    let raw_confirmation_link = create_unconfirmed_subscriber(app).await;
    let mut confirmation_link = Url::parse(&raw_confirmation_link).unwrap();
    confirmation_link.set_port(Some(app.port)).unwrap();

    reqwest::get(confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/v3/mail/send"))
        .and(method(http::Method::POST))
        .respond_with(ResponseTemplate::new(200))
        // .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = json!({
            "title": "Newsletters title",
            "content": {
                "text": "plain text",
                "html": "<h1>Hello<h1>",
            }
    });
    app.post_newsletters(&newsletter_request_body).await;
}

#[tokio::test]
async fn newsletters_return_400_for_invalid_data() {
    let app = spawn_app().await;
    let test_cases = vec![
        (json!({}), "empty json"),
        (
            json!({"content": {"text": "text", "html": "<h1>html</h1>"}}),
            "missing title",
        ),
        (json!({"title": "Newsletters"}), "missing content"),
    ];
    for t in test_cases {
        let rs = app.post_newsletters(&t.0).await;
        assert_eq!(
            rs.status(),
            http::StatusCode::BAD_REQUEST,
            "The API did not failed with status 400: {}",
            t.1
        );
    }
}
