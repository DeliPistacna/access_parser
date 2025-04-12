use reqwest::Client;
use serde::Serialize;
use serde_json::to_string;

pub struct SlackWebhook {
    webhook_url: String,
}

impl SlackWebhook {
    pub fn new(webhook_url: String) -> Self {
        Self { webhook_url }
    }
    pub async fn send_message(&self, message: Message) -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new();
        let json_message = to_string(&message)?;

        let res = client
            .post(self.webhook_url.clone())
            .header("Content-Type", "application/json") // Set the content type to JSON
            .body(json_message)
            .send()
            .await?;

        if res.status().is_success() {
            println!("Message sent successfully!");
        } else {
            eprintln!("Failed to send message: {}", res.status());
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct Message {
    text: String,
}
impl Message {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
        }
    }
}
