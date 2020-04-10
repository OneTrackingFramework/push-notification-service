use super::schema::puser;

#[derive(Debug, Queryable, Identifiable, Associations)]
#[table_name = "puser"]
pub struct User {
    pub id: i32,
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Insertable)]
#[table_name = "puser"]
pub struct NewUser<'a> {
    pub user_id: &'a str,
    pub token: &'a str,
}
