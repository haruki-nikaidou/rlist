use crate::vfs::combine::CombinableVfsDir;
mod onedrive;
/// # OneDrive Driver
/// To use onedrive as a VFS, you need to provide a refresh token, a client id and a client secret. (*refer to `OnedriveConfig` in `config_struct.rs`*)
pub(crate) use onedrive::OneDriveDriver;

/// # Cloud Driver
/// The cloud driver is a driver that can be used to access a cloud storage service, then use the cloud storage service as a virtual file system(VFS).
#[async_trait::async_trait]
pub trait CloudDriver<Config: Send + Sync> {
    /// Convert the driver into VFS directory.
    fn into_combinable(self) -> CombinableVfsDir;
    async fn new(config: &Config) -> Result<Self, String> where Self: Sized;
}