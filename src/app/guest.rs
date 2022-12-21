use super::disk::Disk;
use super::network_interface::NetworkInterface;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Guest {
    pub description: String,
    pub ip_address: String,
    pub memory: i64,
    pub cores: i64,
    pub spice_port: i64,
    #[serde(default = "default_host")]
    pub host: String,
    pub monitor_socket_path: PathBuf,
    pub pidfile_path: PathBuf,
    pub network_interfaces: Vec<NetworkInterface>,
    pub disks: Vec<Disk>,
}

fn default_host() -> String {
    "host".to_string()
}
