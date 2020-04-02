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

use messaging::firebase::*;
use db::models::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;

fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url));
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

    info!("Shutting down");
    Ok(())
}
