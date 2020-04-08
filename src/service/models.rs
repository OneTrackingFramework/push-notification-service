use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum DeviceType {
    IOS,
    FIREBASE, // Can be Android and iOS
}

#[derive(Deserialize)]
pub struct CreateMapping {
    pub userId: String,
    pub deviceToken: String,
    pub deviceType: DeviceType,
}

#[derive(Deserialize)]
pub struct DeleteMapping {
    pub userId: String,
    pub devieToken: String,
}

#[derive(Deserialize)]
pub struct SendMessage {
    pub userId: String,
    pub title: String,
    pub body: String,
}
