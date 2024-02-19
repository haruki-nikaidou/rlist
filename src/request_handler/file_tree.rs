use std::future::Future;
use std::sync::Arc;
use actix_web::{HttpRequest, Responder};
use crate::request_handler::Handler;
use crate::service::drive_whell::DriveWheel;
use crate::vfs::hide_url::UrlHiddenDir;

/// ## Get File Tree Handler
/// path: `/api/v1/file_tree`
pub fn get_file_tree_handler(conf: Arc<DriveWheel>) -> impl Handler<Option<UrlHiddenDir>, dyn Future<Output=Option<UrlHiddenDir>>> {
    move |req: HttpRequest| {
        let conf = conf.clone();
        Some(conf.hidden_url.read())
    }
}