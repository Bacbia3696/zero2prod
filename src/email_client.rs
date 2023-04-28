use reqwest::Client;
use serde::Serialize;

use crate::domain::SubscriberEmail;

#[derive(Clone)]
pub struct EmailClient {
    sender: SubscriberEmail,
    base_url: String,
    http_client: Client,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            sender,
            base_url,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn send_email(
        &self,
        _recipient: SubscriberEmail,
        _subject: &str,
        _html_content: &str,
        _text_content: &str,
    ) -> Result<(), String> {
        let url = format!("{}/email", self.base_url);
        dbg!(&url);
        let req_body = SendEmailRequest {
            personalizations: vec![To {
                to: vec![Receiver {
                    email: "bacbia@gmail.com".to_string(),
                }],
            }],
            from: From {
                email: "asd@gmail.com".to_string(),
            },
            subject: "subject".to_string(),
            content: vec![Content {
                r#type: "text/plain".to_string(),
                value: "kajsdnaksjd".to_string(),
            }],
        };
        let _builder = self.http_client.post(&url).json(&req_body).bearer_auth(
            "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".to_string(),
        );
        _builder.send().await.unwrap();
        Ok(())
    }
}

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
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake,
    };
    use wiremock::{matchers::any, Mock, MockServer, ResponseTemplate};

    use super::*;

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;
    }
}
