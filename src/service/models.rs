use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum DeviceType {
    IOS,
    FIREBASE, // Can be Android and iOS
}

#[derive(Debug, Deserialize)]
pub struct CreateUserData {
    pub user_id: String,
    pub device_token: String,
    pub device_type: DeviceType,
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
