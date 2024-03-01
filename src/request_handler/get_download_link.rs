use actix_web::{Error, get, HttpRequest, HttpResponse, web};
use actix_web::web::Query;
use crate::side_effects::{SideEffect, SideEffectProps};
use crate::State;
use crate::vfs::path_compress::TryPathResult::{*};
use crate::vfs::VfsFile;

#[derive(serde::Deserialize)]
pub struct CaptchaQuery {
    pub token: String
}

/// # Get Download Link API
/// User must provide a valid token(as captcha) to get the download link.
/// If captcha is enabled, user must provide `?token=xxx` to get the download link.
#[get("/api/download/{path:.*}")]
pub async fn get_download_link(
    state: web::Data<State>,
    path: web::Path<(String,)>,
    query: Query<CaptchaQuery>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let token = query.token.as_str();
    let path = path.0.as_str();
    let log_effect = state.log.clone();
    let connection_info = req.connection_info();
    let ip = match connection_info.realip_remote_addr() {
        None => {
            return Ok(HttpResponse::BadRequest().finish());
        }
        Some(ip) => { ip }
    };
    let ua = match req.headers().get("User-Agent") {
        None => "",
        Some(ua) => ua.to_str().unwrap_or("")
    };
    log_effect.do_effect(SideEffectProps {
        request_ip: ip.to_string(),
        user_agent: ua.to_string(),
        file_name: path.to_string()
    }).await;
    let verify = state.captcha.clone();
    if !verify.verify(token, ip).await {
        return Ok(HttpResponse::Unauthorized().finish());
    }
    let wheel = state.wheel.clone();
    let path_map = wheel.get_path_map();
    let file = path_map.try_path(path);
    match file {
        NotFound => {
            Ok(HttpResponse::NotFound().finish())
        },
        File(file) => {
            let url = file.on_download();
            Ok(HttpResponse::TemporaryRedirect().append_header(("Location", url)).finish())
        },
        Dir(_) => {
            Ok(HttpResponse::NotAcceptable().finish())
        }
    }
}
