use std::sync::Arc;
use influxdb::{Client, WriteQuery, Timestamp};
use crate::config_loader::config_struct::InfluxConfig;
use crate::side_effects::{SideEffect, SideEffectCustomConfig, SideEffectProps};

pub struct InfluxProps {
    client: Arc<Client>,
}

async fn write_log(props: SideEffectProps) {
    let user_ip = props.request_ip;
    let user_agent = props.user_agent;
    let file_name = props.file_name;
    let InfluxProps { client } = match props.custom_config {
        SideEffectCustomConfig::Influx(influx_props) => {
            influx_props
        }
        _ => return
    };
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

async fn empty_log_effect(_props: SideEffectProps) {}

pub fn load_log_effect(config: Option<InfluxConfig>) -> SideEffect {
    match config {
        Some(config) => {
            let client = Arc::new(connect_to_influx(config));
            Box::new(
                move |props| {
                    Box::pin(write_log(SideEffectProps{
                        request_ip: props.request_ip,
                        user_agent: props.user_agent,
                        file_name: props.file_name,
                        custom_config: SideEffectCustomConfig::Influx(InfluxProps {
                            client: client.clone(),
                        }),
                    }))
                }
            )
        }
        None => {
            Box::new(
                |props| {
                    Box::pin(empty_log_effect(props))
                }
            )
        }
    }
}