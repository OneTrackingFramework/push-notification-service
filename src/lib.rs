mod db;

use std::error::Error;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use db::models::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
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
    Ok(())
}
