use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserData {
    pub user_id: String,
    pub device_token: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteUserData {
    pub user_id: String,
    pub device_token: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageData {
    pub user_id: String,
    pub title: String,
    pub body: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize_create_user_data() {
        let json = r#"
            {
                "userId": "userXY",
                "deviceToken": "deviceXY"
            }
        "#;
        assert_eq!(
            serde_json::from_str::<CreateUserData>(json).unwrap(),
            CreateUserData {
                user_id: "userXY".to_owned(),
                device_token: "deviceXY".to_owned(),
            }
        );
    }

    #[test]
    fn test_deserialize_delete_user_data() {
        let json = r#"
            {
                "userId": "userXY",
                "deviceToken": "deviceXY"
            }
        "#;
        assert_eq!(
            serde_json::from_str::<DeleteUserData>(json).unwrap(),
            DeleteUserData {
                user_id: "userXY".to_owned(),
                device_token: "deviceXY".to_owned(),
            }
        );
    }

    #[test]
    fn test_deserialize_send_message() {
        let json = r#"
            {
                "userId": "userXY",
                "title": "My title",
                "body": "Text"
            }
        "#;
        assert_eq!(
            serde_json::from_str::<SendMessageData>(json).unwrap(),
            SendMessageData {
                user_id: "userXY".to_owned(),
                title: "My title".to_owned(),
                body: "Text".to_owned(),
            }
        );
    }
}
