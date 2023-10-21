//! src/email_client.rs

use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

/// Our REST email client which talks to an email API provider
///
/// `EmailClient` is really a back-end, as it is a part of our web backend service.
/// It is a client, though, toward a REST API email provider,
/// whichever one we decide to use for sending emails to our subscribers.
///
/// So, we use `EmailClient` to trigger sending of emails from an email provider
/// service to our subscribers.
///
/// `EmailClient` consists of:
///  - `http_client: reqwest::Client` - a new instance of a `reqwest::Client`;
///  - `base_url: String` - the email provider's REST API URL in production,
///     or `localhost` for development purposes;
///  - `sender: SubscriberEmail` - a valid email address that is registered with
///     the email provider and which we use to send emails from;
///  - `authorization_token: Secret<String>` - wrapped in `secrecy::Secret`
///     because we don't want to log this by accident.
///
/// Create an instance of an `EmailClient` through the `new` function,
/// and then send emails through the instance's `send_email` method.
#[derive(Clone, Debug)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: Secret<String>,
}

impl EmailClient {
    /// Constructs a new `EmailClient` which is used for triggering of sending
    /// emails to our subscribers from an email provider.
    ///
    /// Parameters:
    ///  - `base_url: String` - the email provider's REST API URL in production,
    ///     or `localhost` for development purposes;
    ///  - `sender: SubscriberEmail` - a valid email address that is registered with
    ///     the email provider and which we use to send emails from;
    ///  - `authorization_token: Secret<String>` - wrapped in `secrecy::Secret`
    ///     because we don't want to log this by accident.
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
    ) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        self: &Self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref().to_string(),
            to: recipient.as_ref().to_string(),
            subject: subject.to_string(),
            html_body: html_content.to_string(),
            text_body: text_content.to_string(),
        };
        self.http_client
            .post(&url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_body: String,
    text_body: String,
}

#[cfg(test)]
mod tests {
    use super::EmailClient;

    use crate::domain::SubscriberEmail;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        //Arrange
        let mock_server = MockServer::start().await;
        let base_url = mock_server.uri();
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(base_url, sender, Secret::new(Faker.fake()));

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .named("Trigger email provider")
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        // We made at least one matching request, the expectation is satisfied.
        // The `MockServer` will shutdown peacefully, without panicking.
    }
}
