use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use serde::Deserialize;
use tokio_stream::{self as stream, StreamExt};
use crate::config_loader::config_struct::OnedriveConfig;

const AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const MY_DRIVE_URL: &str = "https://graph.microsoft.com/v1.0/me/drive";

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    scope: String,
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
struct MyDrive {
    id: String,
}

pub async fn fetch_access_token(config: &OnedriveConfig) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let res = client.post(AUTH_URL)
        .form(&[
            ("client_id", &config.client_id),
            ("refresh_token", &config.refresh_token),
            ("requested_token_use", &"on_behalf_of".to_owned()),
            ("client_secret", &config.client_secret),
            ("grant_type", &"refresh_token".to_owned()),
        ])
        .send().await;
    match res {
        Ok(res) => {
            let body = res.json::<AccessTokenResponse>().await?;
            Ok(body.access_token)
        }
        Err(e) => Err(Box::new(e))
    }
}

pub async fn get_my_od_id(access_token: &str) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let res = client.get(MY_DRIVE_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .send().await;
    match res {
        Ok(res) => {
            let body = res.json::<MyDrive>().await?;
            Ok(body.id)
        }
        Err(e) => Err(Box::new(e))
    }
}

#[derive(Debug, Deserialize)]
struct ResponseItem {
    id: String,
    name: String,
    size: i64,
    #[serde(rename = "@microsoft.graph.downloadUrl")]
    file_download_url: Option<String>,
    file: Option<String>,
    folder: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResponseList {
    value: Vec<ResponseItem>,
}

async fn request_list(dir_id: String, token: &str) -> Result<ResponseList, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let res = client.get(request_list_url(&dir_id, token)).send().await;
    match res {
        Ok(res) => {
            let body = res.json::<ResponseList>().await?;
            Ok(body)
        }
        Err(e) => Err(Box::new(e))
    }
}

struct OneDriveFile {
    id: String,
    name: String,
    size: i64,
    download_url: String,
}

struct OneDriveFolder {
    id: String,
    name: String,
    size: i64,
    children: Vec<OneDriveItem>,
}

enum OneDriveItem {
    File(OneDriveFile),
    Folder(OneDriveFolder),
    Unknown,
}

impl ResponseItem {
    pub fn into_item(self) -> OneDriveItem {
        match (self.file, self.folder, self.file_download_url) {
            (Some(_), None, Some(url)) => OneDriveItem::File(OneDriveFile {
                id: self.id,
                name: self.name,
                size: self.size,
                download_url: url,
            }),
            (None, Some(_), None) => OneDriveItem::Folder(OneDriveFolder {
                id: self.id,
                name: self.name,
                size: self.size,
                children: Vec::new(),
            }),
            _ => OneDriveItem::Unknown,
        }
    }
}

fn request_list_url(dir_id: &str, drive_id: &str) -> String {
    format!("https://graph.microsoft.com/v1.0/drives/{}/items/{}/children", drive_id, dir_id)
}

type RequestTreeResult = (OneDriveItem, i64);   // 1st: root, 2nd: error count

pub struct OneDriveTreeBuilder {
    token: String,
    drive_id: String,
}

impl OneDriveTreeBuilder {
    pub async fn build_tree(&self, dir_id: String, name: String, size: i64) -> Pin<Box<dyn Future<Output=RequestTreeResult> + '_>> {
        Box::pin(async move {
            let res = request_list(dir_id.clone(), &self.token).await;
            if res.is_err() {
                return (OneDriveItem::Unknown, 1);
            }
            let list = res.unwrap().value;
            let mut error_count = 0;
            let mut folders = Vec::new();
            let mut files = Vec::new();
            list.into_iter().for_each(|item| {
                let item = item.into_item();
                match item {
                    OneDriveItem::Unknown => error_count += 1,
                    OneDriveItem::File(file) => files.push(file),
                    OneDriveItem::Folder(folder) => folders.push(
                        OneDriveItem::Folder(folder)
                    ),
                }
            });
            let folders = folders.into_iter().map(|folder| {
                let folder = match folder {
                    OneDriveItem::Folder(folder) => folder,
                    _ => panic!("Unexpected item type"),
                };
                self.build_tree(folder.id, folder.name, folder.size)
            }).collect::<Vec<_>>();

            todo!()
        })
    }

    pub fn new(token: String, drive_id: String) -> Self {
        OneDriveTreeBuilder {
            token,
            drive_id,
        }
    }
}