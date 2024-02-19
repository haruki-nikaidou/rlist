mod file_tree;
mod get_download_link;

use std::future::Future;
use actix_web::{HttpRequest, Responder};

trait GetHandler<Conf, Res: Responder, F: Future<Output=Res>>: FnOnce(Conf) -> dyn Fn(HttpRequest) -> Box<F> {}