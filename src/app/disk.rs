use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Disk {
    pub label: String,
    pub path: PathBuf,
    pub size: i64,
}
