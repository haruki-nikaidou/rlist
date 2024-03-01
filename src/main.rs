use std::sync::Arc;
use actix_web::{App, HttpServer, web};
use crate::config_loader::{Config, load_config_file};
use crate::service::captcha::{load_captcha, Verify};
use crate::service::drive_whell::DriveWheel;

mod config_loader;
mod vfs;
mod driver;
mod side_effects;
mod service;
mod request_handler;

#[derive(Clone)]
struct State {
    captcha: Arc<dyn Verify>,
    wheel: Arc<DriveWheel>,
}

#[actix_web::main]
async fn main() {
    let Config { influx, drives, cache, captcha } = load_config_file::load_config().unwrap();
    let refresh_interval = cache.refresh_interval;
    let captcha = load_captcha(captcha);
    let wheel = DriveWheel::new(drives, refresh_interval).await;
    let state = Arc::new(State {
        captcha,
        wheel,
    });
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(request_handler::get_file_tree)
    })
        .bind(("127.0.0.1", 8080)).expect("Can not bind to port 8080")
        .run()
        .await.expect("Server failed to start.");
}