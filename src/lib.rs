#[macro_use]
extern crate log;
extern crate env_logger;

extern crate dotenv;
#[macro_use]
extern crate diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

#[macro_use]
extern crate diesel_migrations;

extern crate kafka;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use std::time::Duration;

mod db;
mod endpoints;
use endpoints::*;

use std::env;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use once_cell::sync::Lazy;

embed_migrations!("./migrations");

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

type PgPool = Pool<ConnectionManager<PgConnection>>;

static DBPOOL: Lazy<PgPool> = Lazy::new(|| {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create database pool");
    info!("Created new database pool");
    pool
});

struct FCMHelper {
    client: fcm::Client,
    api_key: String,
}

impl Default for FCMHelper {
    fn default() -> FCMHelper {
        FCMHelper {
            client: fcm::Client::new(),
            api_key: env::var("FCM_API_KEY").expect("FCM_API_KEY must be set"),
        }
    }
}

static FCMHELPER: Lazy<FCMHelper> = Lazy::new(FCMHelper::default);

fn create_kafka_consumer(config: Config) -> Result<Consumer, kafka::Error> {
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

pub struct Config {
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
        if args.is_empty() {
            return Err("not enough arguments");
        }
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

        Ok(Config {
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

macro_rules! handle_topic {
    ($topic: ident, $ms: expr) => {
        if $ms.topic().eq(stringify!($topic)) {
            for message in $ms.messages() {
                let message = message.value.to_vec();
                tokio::spawn(async move {
                    match serde_json::from_slice::<$topic::Req>(message.as_slice()) {
                        Ok(req) => {
                            if let Err(e) = req.handle().await {
                                warn!("Could not handle request: {}", e);
                            }
                        }
                        Err(e) => warn!("Could not deserialize kafka message: {}", e),
                    }
                });
            }
            true
        } else {
            false
        }
    };
}

pub fn run(config: Config, shutdown: Arc<AtomicBool>) -> Result<(), Error> {
    // Do db migrations
    embedded_migrations::run(&DBPOOL.get().unwrap())?;
    info!("Database migrations completed");

    let no_commit = config.no_commit;
    let mut consumer = create_kafka_consumer(config).unwrap();

    info!("Service started. Listening for events...");

    while !shutdown.load(Ordering::Relaxed) {
        for ms in consumer
            .poll()
            .expect("Failed to poll from Kafka consumer")
            .iter()
        {
            let _ = handle_topic!(push_notification, ms)
                || handle_topic!(create_user_device_mapping, ms)
                || handle_topic!(delete_user_device_mapping, ms);
            let _ = consumer.consume_messageset(ms);
        }
        if !no_commit {
            consumer
                .commit_consumed()
                .expect("Failed to commit consumed message");
        }
    }
    info!("Shutting down");
    Ok(())
}
