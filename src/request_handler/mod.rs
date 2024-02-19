pub mod file_tree;
pub mod get_download_link;

use std::future::Future;
use actix_web::{HttpRequest, Responder};

pub use file_tree::get_file_tree_handler;
pub use get_download_link::get_download_link_handler;
trait Handler<Res: Responder, F: Future<Output=Res>>: Fn(HttpRequest) -> F {}