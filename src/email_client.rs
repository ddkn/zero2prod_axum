use crate::domain::SubscriberEmail;

pub struct EmailClient {
    sender: SubscriberEmail,
}

impl EmailClient {
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_component: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}
