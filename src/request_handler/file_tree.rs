use actix_web::{get, web};
use crate::State;

/// # Get File Tree API
/// Users can only get the file tree **without** download links.
#[get("/api/file_tree")]
async fn get_file_tree(state: web::Data<State>) -> String {
    let dir = state.wheel.get_hidden_url();
    serde_json::to_string(dir.as_ref()).unwrap()
}