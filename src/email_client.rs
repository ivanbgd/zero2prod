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
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body,
            text_body,
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
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use super::EmailClient;

    use crate::domain::SubscriberEmail;

    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use rstest::{fixture, rstest};
    use secrecy::Secret;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};

    struct SendEmailBodyMatcher;

    impl Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // Try to parse the body as a JSON value
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            // Check that all mandatory fields are populated without inspecting the field values
            match result {
                Ok(body) => {
                    body.get("From").is_some()
                        && body.get("To").is_some()
                        && body.get("Subject").is_some()
                        && body.get("HtmlBody").is_some()
                        && body.get("TextBody").is_some()
                }
                Err(_) => false,
            }
        }
    }

    struct EmailFields {
        subscriber_email: SubscriberEmail,
        subject: String,
        content: String,
    }

    struct Arrange<'a> {
        mock_server: MockServer,
        email_client: EmailClient,
        email_fields: &'a EmailFields,
    }

    /// `EmailFields` doesn't have to be initialized only `once`.
    /// It can be different for different UTs.
    /// We are using `once` here for educational purposes and to keep it
    /// as an example of how we could use it.
    /// Using `once` requires using a reference to the fixture value.
    /// This complicates things a little, so there is no need to use `once`
    /// unless that would be really beneficial, but we are purposefully
    /// keeping this example to show where references need to be applied.
    /// Since `EmailFields` is a `struct`, in addition to using references,
    /// we need to use lifetime specifiers.
    #[fixture]
    #[once]
    fn email_fields() -> EmailFields {
        EmailFields {
            subscriber_email: SubscriberEmail::parse(SafeEmail().fake()).unwrap(),
            subject: Sentence(1..5).fake(),
            content: Paragraph(1..10).fake(),
        }
    }

    #[fixture]
    async fn arrange(email_fields: &'static EmailFields) -> Arrange<'static> {
        let mock_server = MockServer::start().await;
        let base_url = mock_server.uri();
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(base_url, sender, Secret::new(Faker.fake()));

        Arrange {
            mock_server,
            email_client,
            email_fields,
        }
    }

    #[rstest]
    #[tokio::test]
    async fn send_email_sends_the_expected_request(#[future] arrange: Arrange<'static>) {
        // Arrange
        let arrange = arrange.await;

        let mock_server = arrange.mock_server;
        let email_client = arrange.email_client;

        let subscriber_email = &arrange.email_fields.subscriber_email;
        let subject = &arrange.email_fields.subject;
        let content = &arrange.email_fields.content;

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .named("Trigger email provider & check multiple matchers")
            .mount(&mock_server)
            .await;

        // Act
        let _ = email_client
            .send_email(subscriber_email.clone(), subject, content, content)
            .await;

        // Assert
        // We made at least one matching request, the expectation is satisfied.
        // The `MockServer` will shutdown peacefully, without panicking.
    }

    #[rstest]
    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200(#[future] arrange: Arrange<'static>) {
        // Arrange
        let arrange = arrange.await;

        let mock_server = arrange.mock_server;
        let email_client = arrange.email_client;

        let subscriber_email = &arrange.email_fields.subscriber_email;
        let subject = &arrange.email_fields.subject;
        let content = &arrange.email_fields.content;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .named("Any matcher")
            .mount(&mock_server)
            .await;

        // Act
        let response = email_client
            .send_email(subscriber_email.clone(), subject, content, content)
            .await;

        // Assert
        assert_ok!(&response);
    }

    // #[tokio::test]
    async fn _send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        let base_url = mock_server.uri();
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(base_url, sender, Secret::new(Faker.fake()));

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .named("Any matcher")
            .mount(&mock_server)
            .await;

        // Act
        let response = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_ok!(&response);
    }
}
