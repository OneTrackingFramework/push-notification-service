extern crate env_logger;
extern crate log;

use dotenv::dotenv;
use std::env;
use std::process;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use pushy::*;

#[tokio::main]
async fn main() {
    let shutdown: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let handler_shutdown = shutdown.clone();
    ctrlc::set_handler(move || {
        handler_shutdown.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    dotenv().ok();
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = run(config, shutdown) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
