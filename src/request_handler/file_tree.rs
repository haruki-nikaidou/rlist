use std::future::Future;
use std::sync::Arc;
use actix_web::{HttpRequest, Responder};
use crate::service::drive_whell::DriveWheel;
use crate::vfs::hide_url::UrlHiddenDir;

/// ## Get File Tree Handler
/// path: `/api/v1/file_tree`
pub fn get_file_tree_handler(conf: Arc<DriveWheel>) -> impl Fn(HttpRequest) -> Box<dyn Future<Output=UrlHiddenDir>> {
    move |req: HttpRequest| {
        let conf = conf.clone();
        Box::new(async move {
            conf.hidden_url.read().await.clone()
        })
    }
}