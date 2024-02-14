use std::cell::UnsafeCell;
use std::sync::{Arc};
use std::time::Duration;
use tokio::sync::{RwLock};
use tokio::time::interval;
use tracing::error;
use crate::config_loader::config_struct::DriveConfig;
use crate::driver::CloudDriver;
use crate::driver::onedrive::{OneDriveDriver};
use crate::vfs::combine::{CombinableVfsDir, combine_vfs_dirs};
use crate::vfs::hide_url::{hide_url_for_dir, UrlHiddenDir};

pub struct DriveWheel {
    pub full: Arc<RwLock<CombinableVfsDir>>,
    pub hidden_url: Arc<RwLock<UrlHiddenDir>>,
    drive_config: Vec<DriveConfig>,
    stop_signal: UnsafeCell<StopSignal>,
}

unsafe impl Send for DriveWheel {}
unsafe impl Sync for DriveWheel {}

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
    return combine_vfs_dirs(drives);
}

impl DriveWheel {
    async fn refresh(full: Arc<RwLock<CombinableVfsDir>>, hidden_url: Arc<RwLock<UrlHiddenDir>>, drive_config: &Vec<DriveConfig>) {
        let vfs = get_vfs(drive_config).await;
        let hidden = hide_url_for_dir(&vfs);
        *full.write().await = vfs;
        *hidden_url.write().await = hidden;
    }
    pub async fn new(drive_config: Vec<DriveConfig>, refresh_time: i64) -> Arc<DriveWheel> {
        let vfs = get_vfs(&drive_config).await;
        let hidden = hide_url_for_dir(&vfs);
        let full = Arc::new(RwLock::new(vfs));
        let hidden_url = Arc::new(RwLock::new(hidden));
        let stop_signal = StopSignal::new();
        let instance = Arc::new(DriveWheel {
            full,
            hidden_url,
            drive_config,
            stop_signal,
        });
        let instance_clone = instance.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(refresh_time as u64));
            loop {
                interval.tick().await;
                if unsafe {
                    match instance.stop_signal.get().as_ref() {
                        Some(signal) => signal.is_stop(),
                        None => true
                    }
                } {
                    break;
                }
                Self::refresh(instance.full.clone(), instance.hidden_url.clone(), &instance.drive_config).await;
            }
        });
        return instance_clone;
    }
}

struct StopSignal {
    stop: bool
}

impl StopSignal {
    pub fn new() -> UnsafeCell<Self> {
        UnsafeCell::new(StopSignal {
            stop: false
        })
    }
    pub fn is_stop(&self) -> bool {
        self.stop
    }
    pub fn stop(&mut self) {
        self.stop = true;
    }
}