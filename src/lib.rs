#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate dotenv;
use db::models::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;

extern crate kafka;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};

mod db;
mod messaging;
mod service;

use std::env;
use std::error::Error;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

use futures::executor::ThreadPool;

use service::jwe_helper::JWEHelper;
use service::models::{CreateUserData, DeleteUserData, SendMessageData};

use messaging::client::MessagingClient;
use messaging::fcm_helper::FirebaseCloudMessaging;

embed_migrations!("./migrations");

fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    info!("Established database connection");
    connection
}

fn create_kafka_consumer(config: Config) -> Result<Consumer, Box<dyn Error>> {
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
    jwe_secret: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.is_empty() {
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
        if env::var("KAFKA_FALLBACK_OFFSET").is_ok() {
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
        let no_commit = commit.eq_ignore_ascii_case("TRUE");
        let jwe_secret = env::var("JWE_SECRET").expect("JWE_SECRET must be set");
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
            jwe_secret,
        })
    }
}

pub async fn create_user_device_mapping<'a>(
    connection: Arc<Mutex<PgConnection>>,
    create_user_data: CreateUserData,
) {
    use db::schema::puser::dsl::*;

    let new_user = NewUser {
        user_id: &create_user_data.user_id,
        token: &create_user_data.device_token,
    };
    match diesel::insert_into(puser)
        .values(&new_user)
        .get_result::<User>(&*connection.lock().unwrap())
    {
        Err(e) => warn!(
            "Could not create mapping {:?}, b/c: {}",
            create_user_data, e
        ),
        Ok(_) => debug!("Created mapping: {:?}", create_user_data),
    }
}

pub async fn delete_user_device_mapping<'a>(
    connection: Arc<Mutex<PgConnection>>,
    delete_user_data: DeleteUserData,
) {
    use db::schema::puser::dsl::*;

    if diesel::delete(puser.filter(token.eq(&delete_user_data.devie_token)))
        .execute(&*connection.lock().unwrap())
        .is_ok()
    {
        debug!("Deleted mapping: {:?}", delete_user_data);
    } else {
        warn! {"Could not delete mapping: {:?}", delete_user_data};
    }
}

pub async fn send_messages_to_user<'a>(
    connection: Arc<Mutex<PgConnection>>,
    fcm_client: Arc<FirebaseCloudMessaging>,
    send_message_data: SendMessageData,
) {
    use db::schema::puser::dsl::*;
    let users: Result<Vec<User>, diesel::result::Error> = db::schema::puser::table
        .filter(user_id.eq(&send_message_data.user_id))
        .load(&*connection.lock().unwrap());
    if let Ok(users) = users {
        for user in users {
            if let Err(e) = fcm_client
                .send(
                    &user.token,
                    &send_message_data.title,
                    &send_message_data.body,
                )
                .await
            {
                warn!("Could not send message {:?}, b/c: {}", send_message_data, e);
            }
        }
    } else {
        warn!(
            "Could not send message {:?}, b/c could not retrieve user from database",
            send_message_data
        );
    }
}

pub fn run(config: Config, shutdown: Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
    let connection = Arc::new(Mutex::new(establish_connection()));

    // Do db migrations
    embedded_migrations::run(&*connection.lock().unwrap())?;
    info!("Database migrations completed");

    let mut consumer = create_kafka_consumer(config.clone()).unwrap();

    let fcm_client = Arc::new(FirebaseCloudMessaging::new(&config.fcm_api_key));

    let jwe_helper = Arc::new(JWEHelper::new(&config.jwe_secret));

    info!("Service started. Listening for events...");

    let pool = ThreadPool::new().unwrap();

    while !shutdown.load(Ordering::Relaxed) {
        for ms in consumer
            .poll()
            .expect("Failed to poll from Kafka consumer")
            .iter()
        {
            for m in ms.messages() {
                let topic = ms.topic().to_owned();
                let token = m.value.to_owned();

                let connection = connection.clone();
                let fcm_client = fcm_client.clone();
                let jwe_helper = jwe_helper.clone();
                pool.spawn_ok(async move {
                    if let Ok(message) = jwe_helper.decrypt(&token) {
                        match &topic[..] {
                            "push-notification" => {
                                if let Ok(send_message_data) =
                                    serde_json::from_str::<SendMessageData>(&message)
                                {
                                    send_messages_to_user(
                                        connection.clone(),
                                        fcm_client,
                                        send_message_data,
                                    )
                                    .await;
                                } else {
                                    warn!("Could deserialize data for sending")
                                }
                            }
                            "create-user-device-mapping" => {
                                if let Ok(create_user_data) =
                                    serde_json::from_str::<CreateUserData>(&message)
                                {
                                    create_user_device_mapping(
                                        connection.clone(),
                                        create_user_data,
                                    )
                                    .await;
                                } else {
                                    warn!("Could deserialize data to create user mapping")
                                }
                            }
                            "delete-user-device-mapping" => {
                                if let Ok(delete_user_data) =
                                    serde_json::from_str::<DeleteUserData>(&message)
                                {
                                    delete_user_device_mapping(connection, delete_user_data).await;
                                } else {
                                    warn!("Could deserialize data to delete user mapping")
                                }
                            }
                            unknown => warn!("Cannot handle unknown topic: {}", unknown),
                        };
                    } else {
                        warn!("Could not decrypt jwe message");
                    }
                });
            }
            let _ = consumer.consume_messageset(ms);
        }
        if !config.no_commit {
            consumer
                .commit_consumed()
                .expect("Failed to commit consumed message");
        }
    }
    info!("Shutting down");
    Ok(())
}
