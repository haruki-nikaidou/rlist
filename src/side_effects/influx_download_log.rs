use std::sync::Arc;
use influxdb::{Client, WriteQuery, Timestamp};

pub fn write_log(bucket_name: &str, client: Arc<Client>, user_ip: String, user_agent: String, file_name: &str) {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let write_query = WriteQuery::new(Timestamp::Seconds(now as u128), "download_log")
        .add_field("user_ip", user_ip)
        .add_field("user_agent", user_agent)
        .add_field("file_name", file_name);
    let write = async move {
        match client.query(&write_query).await {
            _ => {}
        };
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
}
