use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use crate::vfs::combine::CombinableVfsDir;

mod onedrive;

pub struct CreateDriverError {
    pub message: String,
}

pub trait CloudDriver<Config> {
    fn into_combinable(self) -> CombinableVfsDir;
    fn new(config: &Config) -> Pin<Box<dyn Future<Output = Result<Self, Box<dyn Error>>> + '_>> where Self: Sized;
}