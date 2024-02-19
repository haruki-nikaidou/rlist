use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub mod influx_download_log;

pub enum SideEffectCustomConfig {
    Influx(influx_download_log::InfluxProps),
    None,
}

pub struct SideEffectProps {
    pub request_ip: String,
    pub user_agent: String,
    pub file_name: String,
    pub custom_config: SideEffectCustomConfig,
}

pub type SideEffect = Arc<dyn Fn(SideEffectProps) -> Pin<Box<dyn Future<Output=()>>>>;