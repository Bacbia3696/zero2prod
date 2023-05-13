use std::time::Duration;

use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

use crate::domain::SubscriberEmail;

#[derive(Clone)]
pub struct EmailClient {
    sender: SubscriberEmail,
    base_url: String,
    http_client: Client,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            sender,
            base_url,
            http_client: reqwest::Client::builder().timeout(timeout).build().unwrap(),
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/v3/mail/send", self.base_url);
        let req_body = SendEmailRequest {
            personalizations: vec![To {
                to: vec![Receiver {
                    email: recipient.as_ref().to_owned(),
                }],
            }],
            from: From {
                email: self.sender.as_ref().to_owned(),
            },
            subject: subject.to_string(),
            content: vec![
                Content {
                    r#type: "text/plain".to_string(),
                    value: text_content.to_string(),
                },
                Content {
                    r#type: "text/html".to_string(),
                    value: html_content.to_string(),
                },
            ],
        };
        let builder = self
            .http_client
            .post(&url)
            .json(&req_body)
            .bearer_auth(self.authorization_token.expose_secret());
        builder.send().await?.error_for_status()?;
        Ok(())
    }
}

// TODO: make this request use ref instead of owned data
#[derive(Debug, Serialize)]
struct SendEmailRequest {
    pub personalizations: Vec<To>,
    pub from: From,
    pub subject: String,
    pub content: Vec<Content>,
}

#[derive(Debug, Serialize)]
struct To {
    pub to: Vec<Receiver>,
}

#[derive(Debug, Serialize)]
struct Receiver {
    pub email: String,
}

#[derive(Debug, Serialize)]
struct Content {
    r#type: String,
    value: String,
}

#[derive(Debug, Serialize)]
struct From {
    pub email: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use wiremock::{
        matchers::{header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    struct SendEmailBodyMatcher;

    // TODO: update this
    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            // Try to parse the body as a JSON value
            let _result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            true
        }
    }
    /// Generate a random email subject
    fn subject() -> String {
        Sentence(1..2).fake()
    }
    /// Generate a random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }
    /// Generate a random subscriber email
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            Secret::new(Faker.fake()),
            std::time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists(http::header::AUTHORIZATION))
            .and(header(http::header::CONTENT_TYPE, "application/json"))
            .and(path("/v3/mail/send"))
            .and(method(http::method::Method::POST))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // Act
        let _ = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;
    }

    #[tokio::test]
    async fn send_email_response_success() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists(http::header::AUTHORIZATION))
            .and(header(http::header::CONTENT_TYPE, "application/json"))
            .and(path("/v3/mail/send"))
            .and(method(http::method::Method::POST))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject = subject();
        let content = content();

        // Act
        let res = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;
        assert_ok!(res);
    }

    #[tokio::test]
    async fn send_email_response_error() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists(http::header::AUTHORIZATION))
            .and(header(http::header::CONTENT_TYPE, "application/json"))
            .and(path("/v3/mail/send"))
            .and(method(http::method::Method::POST))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // Act
        let res = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;
        assert_err!(res);
    }

    #[tokio::test]
    async fn send_email_response_timeout() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists(http::header::AUTHORIZATION))
            .and(header(http::header::CONTENT_TYPE, "application/json"))
            .and(path("/v3/mail/send"))
            .and(method(http::method::Method::POST))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(300)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject = subject();
        let content = content();

        // Act
        let res = email_client
            .send_email(&subscriber_email, &subject, &content, &content)
            .await;
        assert_err!(res);
    }
}
