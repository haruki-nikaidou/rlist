use influxdb::Client;
use crate::config_loader::{Config, load_config_file};
use crate::config_loader::config_struct::InfluxConfig;

mod config_loader;
mod vfs;
mod driver;
mod side_effects;
mod service;

fn main() {
    let Config { influx, drives, cache, captcha } = load_config_file::load_config().unwrap();

    println!("Hello, world!");
}


