use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateUserData {
    pub user_id: String,
    pub device_token: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteUserData {
    pub user_id: String,
    pub devie_token: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageData {
    pub user_id: String,
    pub title: String,
    pub body: String,
}
