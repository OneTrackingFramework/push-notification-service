#[derive(Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub userid: String,
    pub deviceid: i32,
}

#[derive(Debug, Queryable)]
pub struct Device {
    pub id: i32,
    pub devicetypeid: u32,
    pub token: String,
}

#[derive(Debug, Queryable)]
pub struct DeviceType {
    pub id: i32,
    pub name: String,
}
