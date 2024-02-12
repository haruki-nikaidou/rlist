use std::error::Error;
use std::fs::File;
use crate::config_loader::{Config, CONFIG_PATH};
use crate::config_loader::config_struct::{CacheSetting, ConfigFile};

pub fn load_config() -> Result<Config, Box<dyn Error>> {
    let config_file = File::open(CONFIG_PATH)?;
    let config_file: ConfigFile = serde_json::from_reader(config_file)?;

    // set default value for cache
    match config_file.cache {
        None => {
            Ok(Config {
                influx: config_file.influx,
                drives: config_file.drives,
                cache: CacheSetting {
                    refresh_interval: 600,
                }
            })
        },
        Some(cache) => {
            Ok(Config {
                influx: config_file.influx,
                drives: config_file.drives,
                cache
            })
        }
    }
}