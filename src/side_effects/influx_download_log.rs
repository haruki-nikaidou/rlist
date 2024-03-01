use std::pin::Pin;
use std::sync::Arc;
use influxdb::{Client, WriteQuery, Timestamp};
use crate::config_loader::config_struct::InfluxConfig;
use crate::side_effects::{SideEffect, SideEffectProps};

type WriteFunction = Arc<dyn Fn(SideEffectProps) -> Pin<Box<dyn std::future::Future<Output=()> + Send>> + Send + Sync>;

pub struct LogEffect {
    write_fn: WriteFunction
}

async fn log_with_influx(props: SideEffectProps, client: Arc<Client>) {
    let user_ip = props.request_ip;
    let user_agent = props.user_agent;
    let file_name = props.file_name;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let write_query = WriteQuery::new(Timestamp::Seconds(now as u128), "download_log")
        .add_field("user_ip", user_ip)
        .add_field("user_agent", user_agent)
        .add_field("file_name", file_name);
    match client.query(&write_query).await {
        _ => {}
    };
}

fn connect_to_influx(config: InfluxConfig) -> Client {
    let InfluxConfig { url, database, username, password } = config;
    match (username, password) {
        (Some(username), Some(password)) => {
            Client::new(url, database)
                .with_auth(username, password)
        }
        _ => {
            Client::new(url, database)
        }
    }
}

#[async_trait::async_trait]
impl SideEffect<Option<InfluxConfig>> for LogEffect {
    fn new(config: Option<InfluxConfig>) -> Self {
        let client = match config {
            Some(config) => Some(Arc::new(connect_to_influx(config))),
            None => None,
        };
        let write_fn: WriteFunction = if client.is_none() {
            Arc::new(|_| Box::pin(async {}))
        } else {
            Arc::new(move|props| {
                let client_clone = client.clone().unwrap();
                Box::pin(
                    async move {
                        log_with_influx(props, client_clone.clone()).await;
                    }
                )
            })
        };
        LogEffect {
            write_fn,
        }
    }

    async fn do_effect(&self, props: SideEffectProps) {
        (self.write_fn)(props).await;
    }
}