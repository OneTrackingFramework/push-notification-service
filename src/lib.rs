#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate diesel;
extern crate dotenv;

mod db;
mod messaging;

use std::env;

use std::error::Error;

use db::models::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use messaging::firebase::*;

use futures::executor;

fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));
    info!("Established database connection");
    connection
}

pub struct Config;

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 1 {
            return Err("not enough arguments");
        }
        Ok(Config {})
    }
}

#[derive(Debug)]
pub enum DeviceTypeName {
    IOS,
    FIREBASE, // Can be Android and iOS
}

pub fn create_user_device_mapping<'a>(
    connection: &PgConnection,
    new_user_id: &'a str,
    new_token: &'a str,
    dtype_name: DeviceTypeName,
) {
    use db::schema::pdevicetype::dsl::*;
    use db::schema::puser::dsl::*;

    if let Ok(new_devicetype) = pdevicetype
        .filter(name.eq(match dtype_name {
            DeviceTypeName::FIREBASE => "FIREBASE",
            DeviceTypeName::IOS => "IOS",
        }))
        .limit(1)
        .load::<DeviceType>(connection)
    {
        let new_devicetype = &new_devicetype[0]; // B/c limit(1)

        let new_user = NewUser {
            user_id: new_user_id,
            device_type_id: &new_devicetype.id,
            token: new_token,
        };
        match diesel::insert_into(puser)
            .values(&new_user)
            .get_result::<User>(connection)
        {
            Err(e) => warn!(
                "Could not create user / device mapping: {:?}, message: {}",
                new_user, e
            ),
            Ok(created_user) => info!("Created user / device mapping: {:?}", created_user),
        }
    } else {
        warn!("Could not create mapping for user: {}, b/c could not retrieve id for device type: {:?}", new_user_id, dtype_name);
    }
}

pub fn delete_user_device_mapping<'a>(connection: &PgConnection, delete_token: &'a str) {
    use db::schema::puser::dsl::*;

    if let Ok(_) = diesel::delete(puser.filter(token.eq(delete_token))).execute(connection) {
        info!("Deleted user / device mapping for token: {}", delete_token);
    } else {
        warn! {"Could not delete user / device mapping for token: {}", delete_token};
    }
}

pub fn send_messages_to_user<'a>(
    connection: &PgConnection,
    send_user_id: &'a str,
    title: &'a str,
    body: &'a str,
) {
    use db::schema::puser::dsl::*;
    let users: Result<Vec<(User, DeviceType)>, diesel::result::Error> = db::schema::puser::table
        .inner_join(db::schema::pdevicetype::table)
        .filter(user_id.eq(send_user_id))
        .load(connection);
    if let Ok(users) = users {
        for user in users {
            match &user.1.name[..] {
                "FIREBASE" => {
                    if let Err(e) = executor::block_on(send_message(&user.0.token, title, body)) {
                        warn!("Could not execute send message future: {}", e);
                    }
                }
                "IOS" => unimplemented!(),
                unknown => {
                    warn!("No action to send message for device type: {}", unknown);
                }
            }
        }
    } else {
        warn!(
            "Could not send message, b/c could not retrieve user with id: {}",
            send_user_id
        );
    }
}

pub fn run(_config: Config) -> Result<(), Box<dyn Error>> {
    use db::schema::pdevicetype::dsl::*;

    let connection = establish_connection();
    let results = pdevicetype
        .limit(5)
        .load::<DeviceType>(&connection)
        .expect("Error loading user");

    for r in results {
        println!("{:?}", r);
    }

    create_user_device_mapping(&connection, "Max", "MaxTOKEN", DeviceTypeName::IOS);
    delete_user_device_mapping(&connection, "MaxTOKEN");

    info!("Shutting down");
    Ok(())
}
