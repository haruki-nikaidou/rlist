use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::time::SystemTime;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::warn;
use crate::config_loader::config_struct::{OnedriveConfig};
use crate::driver::CloudDriver;
use crate::vfs::combine::{CombinableVfsDir, CombinableVfsFile};
use std::marker::Send;

#[async_trait::async_trait]
impl CloudDriver<OnedriveConfig> for OneDriveDriver {
    fn into_combinable(self) -> CombinableVfsDir {
        self.root.to_combinable()
    }

    async fn new(config: &OnedriveConfig) -> Result<Self, String> {
        let access_token = match fetch_access_token(config).await {
            Ok(token) => token,
            Err(_) => return Err("Failed to fetch access token".to_owned()),
        };
        let drive_id = match get_my_od_id(&access_token).await {
            Ok(id) => id,
            Err(_) => return Err("Failed to get drive id".to_owned()),
        };
        let tree_builder = OneDriveTreeBuilder::new(access_token, drive_id.clone());
        let root = tree_builder.build_tree(
            "root".to_owned(),
            "root".to_owned(),
            0,
            SystemTime::now(),
        ).await;
        let (root, error_count) = root;
        if error_count > 0 {
            warn!("{} errors occurred while building the tree {}", error_count, drive_id);
        }
        Ok(OneDriveDriver {
            root: match root {
                OneDriveItem::Folder(folder) => folder,
                _ => panic!("Unexpected item type"),
            }
        })
    }
}

const AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const MY_DRIVE_URL: &str = "https://graph.microsoft.com/v1.0/me/drive";

#[allow(unused)]
#[derive(Debug, Deserialize)]
/// The response json when request `AUTH_URL`.
struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    scope: String,
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
/// Response json when request `MY_DRIVE_URL`.
struct MyDrive {
    id: String,
}

async fn fetch_access_token(config: &OnedriveConfig) -> Result<String, Box<dyn Error>> {
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

async fn get_my_od_id(access_token: &str) -> Result<String, Box<dyn Error>> {
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
/// the file or folder item in the response json.
struct ResponseItem {
    id: String,
    name: String,
    size: i64,
    #[serde(rename = "@microsoft.graph.downloadUrl")]
    file_download_url: Option<String>,
    file: Option<String>,
    folder: Option<String>,
    #[serde(rename = "lastModifiedDateTime")]
    last_modified_date_time: String,
}

#[derive(Debug, Deserialize)]
/// the response json when request the graphql api.
struct ResponseList {
    value: Vec<ResponseItem>,
}

async fn request_list(drive_id: &str, dir_id: &str, token: &str) -> Result<ResponseList, String> {
    let client = reqwest::Client::new();
    let res = client.get(request_list_url(dir_id, drive_id))
        .header("Authorization", format!("Bearer {}", token))
        .send().await;
    match res {
        Ok(res) => {
            let body = match res.json::<ResponseList>().await {
                Ok(body) => body,
                Err(_) => return Err("Failed to parse response".to_owned()),
            };
            Ok(body)
        }
        Err(_) => Err("Failed to request list".to_owned())
    }
}

/// internal struct to represent a file in onedrive
struct OneDriveFile {
    #[allow(unused)]
    id: String,
    name: String,
    size: i64,
    last_modified: SystemTime,
    download_url: String,
}
impl OneDriveFile {
    fn to_combinable(self) -> CombinableVfsFile {
        CombinableVfsFile::new(
            vec![self.download_url],
            self.name,
            self.size as u64,
            self.last_modified,
        )
    }
}

/// internal struct to represent a folder in onedrive
struct OneDriveFolder {
    id: String,
    name: String,
    size: i64,
    last_modified: SystemTime,
    children: Vec<OneDriveItem>,
}
impl OneDriveFolder {
    fn to_combinable(self) -> CombinableVfsDir {
        let (files, dirs): (Vec<_>, Vec<_>) = self.children.into_iter().partition(|item| {
            match item {
                OneDriveItem::File(_) => true,
                _ => false,
            }
        });
        let files = files.into_iter().map(|item| {
            match item {
                OneDriveItem::File(file) => file.to_combinable(),
                _ => panic!("Unexpected item type"),
            }
        }).collect();
        let dirs = dirs.into_iter().map(|item| {
            match item {
                OneDriveItem::Folder(folder) => folder.to_combinable(),
                _ => panic!("Unexpected item type"),
            }
        }).collect();
        CombinableVfsDir::new(
            self.name,
            dirs,
            files,
            self.size as u64,
        )
    }
}

/// internal enum to represent a file or a folder in onedrive
enum OneDriveItem {
    File(OneDriveFile),
    Folder(OneDriveFolder),
    Unknown,
}

impl ResponseItem {
    /// convert response item into `OneDriveItem`
    pub fn into_item(self) -> OneDriveItem {
        match (self.file, self.folder, self.file_download_url) {
            (Some(_), None, Some(url)) => OneDriveItem::File(OneDriveFile {
                id: self.id,
                name: self.name,
                size: self.size,
                download_url: url,
                last_modified: DateTime::<Utc>::from(
                    DateTime::parse_from_rfc3339(
                        self.last_modified_date_time.as_str()
                    ).unwrap()
                ).into(),
            }),
            (None, Some(_), None) => OneDriveItem::Folder(OneDriveFolder {
                id: self.id,
                name: self.name,
                size: self.size,
                children: Vec::new(),
                last_modified: DateTime::<Utc>::from(
                    DateTime::parse_from_rfc3339(
                        self.last_modified_date_time.as_str()
                    ).unwrap()
                ).into(),
            }),
            _ => OneDriveItem::Unknown,
        }
    }
}

fn request_list_url(dir_id: &str, drive_id: &str) -> String {
    format!("https://graph.microsoft.com/v1.0/drives/{}/items/{}/children", drive_id, dir_id)
}

type RequestTreeResult = (OneDriveItem, i64);   // 1st: root, 2nd: error count

/// internal struct to build the tree of the onedrive
struct OneDriveTreeBuilder {
    token: String,
    drive_id: String,
}

impl OneDriveTreeBuilder {
    fn build_tree(&self, dir_id: String, name: String, size: i64, last_modified_time: SystemTime) -> Pin<Box<dyn Future<Output=RequestTreeResult> + '_ + Send>> {
        // When use recursive async function, return type must be `Pin<Box<dyn Future<Output=...> + '_ + Send>>`.
        Box::pin(async move {
            // request the graphql api
            let res = request_list(dir_id.as_str(), self.drive_id.as_str(), self.token.as_str()).await;
            if res.is_err() {
                return (OneDriveItem::Unknown, 1);
            }
            let list = res.unwrap().value;

            // initial error count and two vectors to store files and folders
            let mut error_count = 0;
            let mut folders = Vec::new();
            let mut files = Vec::new();

            // divide the items in the list into files and folders
            list.into_iter().for_each(|item| {
                let item = item.into_item();
                match item {
                    OneDriveItem::Unknown => error_count += 1,
                    OneDriveItem::File(file) => files.push(OneDriveItem::File(file)),
                    OneDriveItem::Folder(folder) => folders.push(
                        OneDriveItem::Folder(folder)
                    ),
                }
            });

            // graphql api will only return the id of the folders, so we need to build the tree recursively
            let folders = folders.into_iter().map(|folder| {
                let folder = match folder {
                    OneDriveItem::Folder(folder) => folder,
                    _ => panic!("Unexpected item type"),
                };
                self.build_tree(folder.id, folder.name, folder.size, folder.last_modified)
            }).collect::<Vec<_>>();

            // wait for all the folders to be built
            let folders = futures::future::join_all(folders).await;
            // collect error count
            let folders = folders.into_iter().map(|(folder, count)| {
                error_count += count;
                folder
            }).collect::<Vec<_>>();

            // process files
            let children = files.into_iter().chain(folders.into_iter()).collect();

            // return the result
            (OneDriveItem::Folder(OneDriveFolder {
                id: dir_id,
                name,
                size,
                last_modified: last_modified_time,
                children,
            }), error_count)
        })
    }

    pub fn new(token: String, drive_id: String) -> Self {
        OneDriveTreeBuilder {
            token,
            drive_id,
        }
    }
}

pub struct OneDriveDriver {
    root: OneDriveFolder,
}