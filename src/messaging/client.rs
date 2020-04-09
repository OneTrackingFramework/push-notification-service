use std::error::Error;

use async_trait::async_trait;

#[async_trait]
pub trait MessagingClient {
    async fn send(&self, token: &str, title: &str, body: &str) -> Result<(), Box<dyn Error>>;
}
