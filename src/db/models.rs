use super::schema::pdevicetype;
use super::schema::puser;

#[derive(Debug, Queryable, Identifiable, Associations, AsExpression)]
#[belongs_to(DeviceType)]
#[table_name = "puser"]
pub struct User {
    pub id: i32,
    pub user_id: String,
    pub device_type_id: i32,
    pub token: String,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsExpression)]
#[table_name = "pdevicetype"]
pub struct DeviceType {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Insertable)]
#[table_name = "puser"]
pub struct NewUser<'a> {
    pub user_id: &'a str,
    pub device_type_id: &'a i32,
    pub token: &'a str,
}
