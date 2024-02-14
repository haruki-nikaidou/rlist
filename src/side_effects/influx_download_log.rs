use std::sync::Arc;
use influxdb::{Client, WriteQuery, Timestamp};
use crate::side_effects::{SideEffectCustomConfig, SideEffectProps};

pub struct InfluxProps {
    bucket_name: String,
    client: Arc<Client>,
}

pub async fn write_log(props: SideEffectProps) {
    let user_ip = props.request_ip;
    let user_agent = props.user_agent;
    let file_name = props.file_name;
    let InfluxProps {bucket_name, client} = match props.custom_config {
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
