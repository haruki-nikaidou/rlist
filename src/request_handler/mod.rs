mod file_tree;

use std::future::Future;
use actix_web::{HttpRequest, Responder};

trait GetHandler<Conf, Res: Responder, F: Future<Output=Res>>: FnOnce(Conf) -> dyn Fn(HttpRequest) -> Box<F> {}