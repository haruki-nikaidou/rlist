use actix_web::{App, HttpServer, web};
use crate::config_loader::{Config, load_config_file};
use crate::service::captcha::load_captcha;
use crate::service::drive_whell::DriveWheel;
use crate::side_effects::influx_download_log::load_log_effect;
use crate::request_handler::get_download_link_handler;

mod config_loader;
mod vfs;
mod driver;
mod side_effects;
mod service;
mod request_handler;

#[actix_web::main]
async fn main() {
    let Config { influx, drives, cache, captcha } = load_config_file::load_config().unwrap();
    let log_effect = load_log_effect(influx);
    let refresh_interval = cache.refresh_interval;
    let captcha = load_captcha(captcha);
    let wheel = DriveWheel::new(drives, refresh_interval).await;
    HttpServer::new(|| {
        App::new().route(
            "/api/v1/download_link/{file_path}",
            web::get().to(
                get_download_link_handler(log_effect.clone(), captcha.clone(), wheel.clone())
            ),
        );
    }).bind("127.0.0.1:8080")?
        .run()
        .await.expect("Cannot start server");
}