#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate diesel;
extern crate dotenv;

extern crate kafka;

mod db;
mod messaging;

use std::env;
use std::io::{self, Write};

use std::error::Error;

use db::models::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use messaging::firebase::*;

use futures::executor;

use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use std::time::Duration;

fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));
    info!("Established database connection");
    connection
}

fn create_consumer(config: Config) -> Result<Consumer, Box<dyn Error>> {
        let mut cb = Consumer::from_hosts(config.brokers)
            .with_group(config.group)
            .with_fallback_offset(config.fallback_offset)
            .with_fetch_max_wait_time(Duration::from_secs(config.fetch_max_wait_time))
            .with_fetch_min_bytes(config.fetch_min_bytes)
            .with_fetch_max_bytes_per_partition(config.fetch_max_bytes_per_partition)
            .with_retry_max_bytes_limit(config.retry_max_bytes_limit)
            .with_offset_storage(config.offset_storage)
            .with_client_id("kafka-pushy-consumer".into());
        for topic in config.topics {
            cb = cb.with_topic(topic);
        }
        Ok(cb.create()?)
}

#[derive(Clone)]
pub struct Config {
    fcm_api_key: String,
    topics: Vec<String>,
    brokers: Vec<String>,
    group: String,
    fallback_offset: FetchOffset,
    offset_storage: GroupOffsetStorage,
    fetch_max_wait_time: u64,
    fetch_min_bytes: i32,
    fetch_max_bytes_per_partition: i32,
    retry_max_bytes_limit: i32,
    no_commit: bool,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 1 {
            return Err("not enough arguments");
        }
        let fcm_api_key = env::var("FCM_API_KEY").expect("FCM_API_KEY must be set");
        let topics: Vec<String> = env::var("KAFKA_TOPICS")
            .expect("KAFKA_TOPICS must be set")
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect();
        let brokers: Vec<String> = env::var("KAFKA_BROKERS")
            .expect("KAFKA_BROKERS must be set")
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect();
        let group = env::var("KAFKA_GROUP").expect("KAFKA_GROUP must be set");
        let mut fallback_offset = FetchOffset::Latest;
        if let Ok(_) = env::var("KAFKA_FALLBACK_OFFSET") {
            fallback_offset = FetchOffset::Earliest;
        };
        let storage = env::var("KAFKA_OFFSET_STORAGE").expect("KAFKA_OFFSET_STORAGE must be set");
        let mut offset_storage = GroupOffsetStorage::Kafka;
        if storage.eq_ignore_ascii_case("zookeeper") {
            offset_storage = GroupOffsetStorage::Zookeeper;
        } else if !storage.eq_ignore_ascii_case("kafka") {
            panic!(format!("Unknown offset store: {}", storage));
        }
        let fetch_max_wait_time = u64::from_str_radix(
            &env::var("KAFKA_FETCH_MAX_WAIT_TIME").expect("KAFKA_FETCH_MAX_WAIT_TIME must be set"),
            10,
        )
        .unwrap();
        let fetch_min_bytes = i32::from_str_radix(
            &env::var("KAFKA_FETCH_MIN_BYTES").expect("KAFKA_FETCH_MIN_BYTES must be set"),
            10,
        )
        .unwrap();
        let fetch_max_bytes_per_partition = i32::from_str_radix(
            &env::var("KAFKA_FETCH_MAX_BYTES_PER_PARTITION")
                .expect("KAFKA_FETCH_MAX_BYTES_PER_PARTITION must be set"),
            10,
        )
        .unwrap();
        let retry_max_bytes_limit = i32::from_str_radix(
            &env::var("KAFKA_RETRY_MAX_BYTES_LIMIT")
                .expect("KAFKA_RETRY_MAX_BYTES_LIMIT must be set"),
            10,
        )
        .unwrap();
        let commit = env::var("KAFKA_NO_COMMIT").expect("KAFKA_NO_COMMIT must be set");
        let mut no_commit = false;
        if commit.eq_ignore_ascii_case("TRUE") {
            no_commit = true;
        }
        Ok(Config {
            fcm_api_key,
            topics,
            brokers,
            group,
            fallback_offset,
            offset_storage,
            fetch_max_wait_time,
            fetch_min_bytes,
            fetch_max_bytes_per_partition,
            retry_max_bytes_limit,
            no_commit,
        })
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
    fcm_api_key: &'a str,
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
                    if let Err(e) =
                        executor::block_on(send_message(fcm_api_key, &user.0.token, title, body))
                    {
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

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let connection = establish_connection();

    let mut consumer = create_consumer(config.clone())?;

    create_user_device_mapping(&connection, "Max Mustermann", "12345", DeviceTypeName::IOS);
    //send_messages_to_user(&connection, &config.fcm_api_key, "Max", "Hallo", "Welt");
    delete_user_device_mapping(&connection, "12345");

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut buf = Vec::with_capacity(1024);

    info!("Service started. Listening for events...");
    let do_commit = !config.no_commit;
    loop {
        for ms in consumer.poll()?.iter() {
            for m in ms.messages() {
                // ~ clear the output buffer
                unsafe { buf.set_len(0) };
                // ~ format the message for output
                let _ = write!(buf, "{}:{}@{}:\n", ms.topic(), ms.partition(), m.offset);
                buf.extend_from_slice(m.value);
                buf.push(b'\n');
                // ~ write to output channel
                stdout.write_all(&buf)?;
            }
            let _ = consumer.consume_messageset(ms);
        }
        if do_commit {
            consumer.commit_consumed()?;
        }
    }

    info!("Shutting down");
    Ok(())
}
