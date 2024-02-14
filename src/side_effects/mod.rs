mod influx_download_log;

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

pub type SideEffect = dyn Fn(SideEffectProps) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>;