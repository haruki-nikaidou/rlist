pub mod config_struct;
mod load_config_file;

use crate::config_loader::config_struct::{CacheSetting, CaptchaConfig, DriveConfig, InfluxConfig};

pub const CONFIG_PATH: &str = "config.json";

pub struct Config {
    pub influx: Option<InfluxConfig>,
    pub drives: Vec<DriveConfig>,
    pub cache: CacheSetting,
    pub captcha: Option<CaptchaConfig>,
}