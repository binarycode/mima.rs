use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Disk {
    pub label: String,
    pub path: PathBuf,
    pub size: i64,
    #[serde(default = "default_disk_model")]
    pub model: String,
}

fn default_disk_model() -> String {
    "scsi-hd".to_string()
}
