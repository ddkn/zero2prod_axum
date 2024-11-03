use crate::domain::SubscriberEmail;
use reqwest::Client;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_component: &str,
        text_content: &str,
    ) -> Result<(), String> {
        //todo!()
        // Dummy Ok
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use crate::test_utils::test_utils::ValidEmailFixture;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use proptest::prelude::*;
    use proptest::test_runner::{Config, TestRunner};
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arange
        let mock_server = MockServer::start().await;

        let mut runner = TestRunner::default();
        let strategy = ValidEmailFixture::arbitrary();
        let email_fixture = strategy.new_tree(&mut runner).unwrap().current();
        let sender =
            SubscriberEmail::parse(email_fixture.as_ref().to_string()).unwrap();

        let email_client = EmailClient::new(mock_server.uri(), sender);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let email_fixture = strategy.new_tree(&mut runner).unwrap().current();
        let subscriber_email =
            SubscriberEmail::parse(email_fixture.as_ref().to_string()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
    }
}
