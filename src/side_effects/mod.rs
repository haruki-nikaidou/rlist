use std::future::Future;

pub mod influx_download_log;

pub struct SideEffectProps {
    pub request_ip: String,
    pub user_agent: String,
    pub file_name: String,
}

#[async_trait::async_trait]
trait SideEffect<Config: Send + Sync>: Send + Sync {
    fn new(config: Config) -> Self;
    async fn do_effect(&self, props: SideEffectProps);
}