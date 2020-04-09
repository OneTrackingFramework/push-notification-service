use std::error::Error;

use super::client::MessagingClient;

use async_trait::async_trait;

pub struct FirebaseCloudMessaging {
    api_key: String,
    client: fcm::Client,
}

#[async_trait]
impl MessagingClient for FirebaseCloudMessaging {
    async fn send(&self, token: &str, title: &str, body: &str) -> Result<(), Box<dyn Error>> {
        let mut notification_builder = fcm::NotificationBuilder::new();
        notification_builder.title(title);
        notification_builder.body(body);

        let notification = notification_builder.finalize();

        let mut message_builder = fcm::MessageBuilder::new(&self.api_key, token);
        message_builder.notification(notification);

        let response = self.client.send(message_builder.finalize()).await?;
        info!("FCM client send: {:?}", response);
        Ok(())
    }
}

impl FirebaseCloudMessaging {
    pub fn new(api_key: &str) -> FirebaseCloudMessaging {
        FirebaseCloudMessaging {
            api_key: api_key.to_owned(),
            client: fcm::Client::new(),
        }
    }
}
