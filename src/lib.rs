use std::error::Error;

pub struct Config;

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 1 {
            return Err("not enough arguments");
        }
        Ok(Config{})
    }
}

pub fn run(_config: Config) -> Result<(), Box<dyn Error>> {
    Ok(())
}