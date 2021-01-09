mod commands;

use crate::command::Execute;
use anyhow::Context;
use anyhow::Result;
use chrono::NaiveDateTime;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

#[derive(Deserialize)]
pub struct App {
    pub guests: BTreeMap<String, Guest>,
    pub networks: BTreeMap<String, Network>,
}

#[derive(Deserialize)]
pub struct Disk {
    pub label: String,
    pub path: PathBuf,
    pub size: i64,
}

#[derive(Deserialize)]
pub struct Guest {
    pub description: String,
    pub ip_address: String,
    pub memory: i64,
    pub cores: i64,
    pub spice_port: i64,
    pub monitor_socket_path: PathBuf,
    pub pidfile_path: PathBuf,
    pub network_interfaces: Vec<NetworkInterface>,
    pub disks: Vec<Disk>,
}

#[derive(Deserialize)]
pub struct Network {
    pub bridge_name: String,
}

#[derive(Deserialize)]
pub struct NetworkInterface {
    #[serde(rename = "network")]
    pub network_id: String,
    pub mac_address: String,
    #[serde(default = "default_network_interface_model")]
    pub model: String,
    pub tap_name: String,
}

pub struct Snapshot {
    pub id: String,
    pub timestamp: NaiveDateTime,
}

impl App {
    pub fn new<T>(path: T) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();

        let config = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read configuration from {}", path.display()))?;
        let app = toml::from_str::<Self>(&config)
            .with_context(|| format!("Failed to parse configuration in {}", path.display()))?;

        let mut binaries = vec![
            "ip",
            "pgrep",
            "pkill",
            "qemu-img",
            "qemu-system-x86_64",
            "socat",
        ];
        binaries.retain(|binary| {
            if let Ok(status) = Command::new("which")
                .arg(binary)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
            {
                !status.success()
            } else {
                true
            }
        });
        if !binaries.is_empty() {
            anyhow::bail!("Dependency missing: {}", binaries.join(", "));
        }

        Ok(app)
    }

    pub fn get_disk_snapshots<T>(
        &self,
        guest_id: T,
        disk_id: usize,
    ) -> Result<HashMap<String, Snapshot>>
    where
        T: AsRef<str>,
    {
        let disk = self.get_guest_disk(guest_id, disk_id)?;

        #[derive(Deserialize)]
        struct QemuImgInfo {
            snapshots: Option<Vec<QemuImgSnapshot>>,
        }

        #[derive(Deserialize)]
        struct QemuImgSnapshot {
            name: String,
            #[serde(rename = "date-sec")]
            timestamp_sec: i64,
            #[serde(rename = "date-nsec")]
            timestamp_nsec: u32,
        }

        let snapshots = command_macros::command!(
            qemu-img info
            --force-share
            --output=json
            (disk.path)
        )
        .execute_and_parse_json_output::<QemuImgInfo>()?
        .snapshots
        .unwrap_or_default()
        .into_iter()
        .map(|snapshot| {
            (
                snapshot.name.clone(),
                Snapshot {
                    id: snapshot.name.clone(),
                    timestamp: NaiveDateTime::from_timestamp(
                        snapshot.timestamp_sec,
                        snapshot.timestamp_nsec,
                    ),
                },
            )
        })
        .collect();

        Ok(snapshots)
    }

    pub fn get_guest<T>(&self, guest_id: T) -> Result<&Guest>
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();

        match self.guests.get(guest_id) {
            Some(guest) => Ok(guest),
            None => anyhow::bail!("Unknown guest `{}`", guest_id),
        }
    }

    pub fn get_guest_disk<T>(&self, guest_id: T, disk_id: usize) -> Result<&Disk>
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();

        let disks = self.get_guest_disks(guest_id)?;
        match disks.get(disk_id) {
            Some(disk) => Ok(disk),
            None => anyhow::bail!(
                "Unknown disk `{disk_id}` for guest `{guest_id}`",
                disk_id = disk_id,
                guest_id = guest_id
            ),
        }
    }

    pub fn get_guest_disks<T>(&self, guest_id: T) -> Result<&Vec<Disk>>
    where
        T: AsRef<str>,
    {
        let guest = self.get_guest(guest_id)?;

        Ok(&guest.disks)
    }

    pub fn get_guest_snapshots<T>(&self, guest_id: T) -> Result<HashMap<String, Snapshot>>
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();

        let mut first_disk = true;
        let mut snapshots = HashMap::new();

        let disks = self.get_guest_disks(guest_id)?;
        for disk_id in 0..disks.len() {
            let disk_snapshots = self.get_disk_snapshots(guest_id, disk_id)?;

            if first_disk {
                first_disk = false;
                snapshots = disk_snapshots;
            } else {
                snapshots.retain(|id, snapshot| {
                    if let Some(disk_snapshot) = disk_snapshots.get(id) {
                        let timestamp_difference = snapshot
                            .timestamp
                            .signed_duration_since(disk_snapshot.timestamp)
                            .num_seconds()
                            .abs();

                        timestamp_difference < 300 // 5 minutes
                    } else {
                        false
                    }
                });
            }
        }

        Ok(snapshots)
    }

    pub fn get_network<T>(&self, network_id: T) -> Result<&Network>
    where
        T: AsRef<str>,
    {
        let network_id = network_id.as_ref();

        match self.networks.get(network_id) {
            Some(network) => Ok(network),
            None => anyhow::bail!("Unknown network `{}`", network_id),
        }
    }
}

impl Guest {
    pub fn is_booted(&self) -> Result<bool> {
        if !self.pidfile_path.exists() || !self.monitor_socket_path.exists() {
            return Ok(false);
        }

        let status = command_macros::command!(
            pgrep
            --full
            --pidfile (self.pidfile_path)
            qemu
        )
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

        Ok(status.success())
    }
}

fn default_network_interface_model() -> String {
    "virtio-net-pci-non-transitional".to_string()
}
