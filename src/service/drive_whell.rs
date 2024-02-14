use std::sync::{Arc};
use tokio::sync::RwLock;
use tracing::error;
use crate::config_loader::config_struct::DriveConfig;
use crate::driver::CloudDriver;
use crate::driver::onedrive::{OneDriveDriver};
use crate::vfs::combine::{CombinableVfsDir, combine_vfs_dirs};
use crate::vfs::hide_url::UrlHiddenDir;

pub struct DriveWheel {
    pub full: Arc<RwLock<CombinableVfsDir>>,
    pub hidden_url: Arc<RwLock<UrlHiddenDir>>,
    pub drive_config: Vec<DriveConfig>,
}

async fn get_vfs(drive_config: &Vec<DriveConfig>) -> CombinableVfsDir {
    let drives: Vec<_> = drive_config.iter()
        .map(|config| async move {
            match config {
                DriveConfig::Onedrive(config) => {
                    match OneDriveDriver::new(config).await {
                        Ok(driver) => Some(driver.into_combinable()),
                        Err(e) => {
                            error!("Failed to create driver: {}", e);
                            None
                        }
                    }
                }
            }
        }).collect();
    let drives = futures::future::join_all(drives).await;
    let drives: Vec<_> = drives.into_iter().filter_map(|x| x).collect();
    return  combine_vfs_dirs(drives);
}
