use super::db::schema::puser;
use super::*;
use serde::{Deserialize, Serialize};

pub mod create_user_device_mapping {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, Insertable)]
    #[table_name = "puser"]
    #[serde(rename_all = "camelCase")]
    pub struct Req<'a> {
        user_id: &'a str,
        device_token: &'a str,
    }

    impl<'a> Req<'a> {
        pub async fn handle(self) -> Result<(), Error> {
            use db::schema::puser::dsl::*;
            diesel::insert_into(puser)
                .values(self)
                .execute(&DBPOOL.get()?)?;
            Ok(())
        }
    }
}

pub mod delete_user_device_mapping {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Req<'a> {
        user_id: &'a str,
        device_token: &'a str,
    }

    impl<'a> Req<'a> {
        pub async fn handle(self) -> Result<(), Error> {
            use db::schema::puser::dsl::*;
            diesel::delete(puser.filter(device_token.eq(&self.device_token)))
                .execute(&DBPOOL.get()?)?;
            Ok(())
        }
    }
}

pub mod push_notification {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Req<'a> {
        user_id: &'a str,
        title: &'a str,
        body: &'a str,
    }

    #[derive(Debug, Queryable)]
    struct User {
        id: i32,
        user_id: String,
        device_token: String,
    }

    fn build_message<'a>(token: &'a str, title: &'a str, body: &'a str) -> fcm::Message<'a> {
        let mut notification_builder = fcm::NotificationBuilder::new();
        notification_builder.title(title);
        notification_builder.body(body);

        let notification = notification_builder.finalize();

        let mut message_builder = fcm::MessageBuilder::new(&FCMHELPER.api_key, token);
        message_builder.notification(notification);
        message_builder.finalize()
    }

    impl<'a> Req<'a> {
        pub async fn handle(self) -> Result<(), Error> {
            use db::schema::puser::dsl::*;

            let users: Vec<User> = db::schema::puser::table
                .filter(user_id.eq(self.user_id))
                .load(&DBPOOL.get()?)?;
            for user in users {
                FCMHELPER
                    .client
                    .send(build_message(&user.device_token, &self.title, &self.body))
                    .await?;
            }
            Ok(())
        }
    }
}
