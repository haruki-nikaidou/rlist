use std::future::Future;
use std::sync::Arc;
use actix_web::{HttpRequest, web};
use serde::Deserialize;
use crate::service::captcha::Verify;
use crate::service::drive_whell::DriveWheel;
use crate::side_effects::{SideEffect, SideEffectCustomConfig, SideEffectProps};
use crate::vfs::path_compress::TryPathResult;
use crate::vfs::VfsFile;

pub enum GetDownloadLinkResult {
    Ok(String),
    Err(GetDownloadLinkError),
}

pub enum GetDownloadLinkError {
    InvalidCaptcha,
    FileNotFound,
    InvalidRequest,
}

#[derive(Debug, Deserialize)]
pub struct CaptchaQuery {
    pub captcha: String,
}

/// Get the real download link
/// path: `/api/v1/download_link/{file_path}?captcha={captcha}`
pub fn get_download_link_handler(log: SideEffect, captcha: Arc<dyn Verify>, wheel: Arc<DriveWheel>) ->
impl Fn(HttpRequest) -> Box<dyn Future<Output=GetDownloadLinkResult>> {
    move |req: HttpRequest| {
        let ip = req.connection_info().realip_remote_addr().unwrap().to_string();
        let file_path = match req.match_info().get("file_path") {
            Some(file_path) => file_path.to_string(),
            None => return Box::new(async {
                return GetDownloadLinkResult::Err(GetDownloadLinkError::InvalidRequest)
            }),
        };
        let query = web::Query::<CaptchaQuery>::from_query(
            req.query_string()
        );
        let captcha_query = match query {
            Ok(captcha_query) => captcha_query.captcha.clone(),
            Err(_) => "".to_string(),
        };
        let captcha = captcha.clone();
        let wheel = wheel.clone();
        let log = log.clone();
        Box::new(
            async move {
                log(SideEffectProps{
                    request_ip: ip.clone(),
                    user_agent: req.headers().get("User-Agent").unwrap().to_str().unwrap().to_string(),
                    file_name: file_path.clone(),
                    custom_config: SideEffectCustomConfig::None,
                }).await;
                let verify_future = captcha.verify(captcha_query.as_str(), ip.as_str()).await;
                if !verify_future {
                    return GetDownloadLinkResult::Err(GetDownloadLinkError::InvalidCaptcha)
                }
                let path_map = wheel.path_map.read().await;
                match path_map.try_path(file_path.as_str()) {
                    TryPathResult::NotFound => GetDownloadLinkResult::Err(GetDownloadLinkError::FileNotFound),
                    TryPathResult::Dir(_) => GetDownloadLinkResult::Err(GetDownloadLinkError::FileNotFound),
                    TryPathResult::File(file) => GetDownloadLinkResult::Ok(file.on_download()),
                }
            }
        )
    }
}