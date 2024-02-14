use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::vfs::combine::CombinableVfsDir;

pub mod onedrive;

pub trait CloudDriver<Config: Send + Sync> {
    fn into_combinable(self) -> CombinableVfsDir;
    fn new(config: &Config) -> Pin<Box<dyn Future<Output = Result<Self, String>> + '_ + Send>> where Self: Sized;
}