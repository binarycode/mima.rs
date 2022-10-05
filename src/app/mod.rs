mod commands;
mod disk;
mod guest;
mod guest_connection;
mod host_connection;
mod network;
mod network_interface;
mod snapshot;

use crate::command::Execute;
use crate::errors::ForbiddenRemoteExecutionError;
use crate::errors::MissingDependenciesError;
use crate::errors::MissingIPAddressConfigurationError;
use crate::errors::ParseConfigurationError;
use crate::errors::ProcessExecutionError;
use crate::errors::ReadConfigurationError;
use crate::errors::UnknownGuestError;
use crate::errors::UnknownNetworkError;
use anyhow::Result;
use disk::Disk;
use guest::Guest;
use guest_connection::GuestConnection;
use host_connection::HostConnection;
use network::Network;
use serde::Deserialize;
use snapshot::Snapshot;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

const GUEST_WORKSPACE_PATH: &str = "/root/mima";
pub const SSH_CONNECTION_TIMEOUT: u64 = 100;

#[derive(Deserialize)]
pub struct App {
    pub guests: BTreeMap<String, Guest>,
    pub networks: BTreeMap<String, Network>,

    #[serde(skip_deserializing)]
    host_connection: Option<HostConnection>,
}

impl App {
    pub fn new<T>(path: T, host: Option<String>) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();

        let config =
            std::fs::read_to_string(path).map_err(|_| ReadConfigurationError::new(path))?;

        let mut app =
            toml::from_str::<Self>(&config).map_err(|_| ParseConfigurationError::new(path))?;
        if let Some(host) = host {
            app.host_connection = Some(HostConnection::new(host)?);
        }
        let app = app;

        let mut binaries = vec![
            "ip",
            "pgrep",
            "pkill",
            "qemu-img",
            "qemu-system-x86_64",
            "socat",
        ];
        binaries.retain(|binary| {
            let which = app.prepare_host_command("which");
            let status = command_macros::command! {
                { which } (binary)
            }
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

            if let Ok(status) = status {
                !status.success()
            } else {
                true
            }
        });
        if !binaries.is_empty() {
            anyhow::bail!(MissingDependenciesError::new(binaries));
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
        // we receive disk by index, not by label, so we can be sure it exists
        // so we can use unwrap here
        let disk = self.get_guest_disks(guest_id)?.get(disk_id).unwrap();

        #[derive(Deserialize)]
        struct QemuImgInfo {
            snapshots: Option<Vec<QemuImgSnapshot>>,
        }

        #[derive(Deserialize)]
        struct QemuImgSnapshot {
            name: String,
            #[serde(rename = "date-sec")]
            timestamp_sec: u64,
            #[serde(rename = "date-nsec")]
            timestamp_nsec: u32,
        }

        let qemu_img = self.prepare_host_command("qemu-img");
        let snapshots = command_macros::command! {
            {qemu_img} info --force-share --output=json (disk.path)
        }
        .execute_and_parse_json_output::<QemuImgInfo>()?
        .snapshots
        .unwrap_or_default()
        .into_iter()
        .map(|snapshot| {
            (
                snapshot.name.clone(),
                Snapshot {
                    id: snapshot.name.clone(),
                    timestamp: Duration::new(snapshot.timestamp_sec, snapshot.timestamp_nsec),
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
            None => anyhow::bail!(UnknownGuestError::new(guest_id)),
        }
    }

    pub fn get_guest_connection<T>(
        &self,
        guest_id: T,
        max_connection_timeout: u64,
    ) -> Result<GuestConnection>
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();

        let guest = self.get_guest(guest_id)?;

        match &guest.ip_address {
            Some(ip_address) => GuestConnection::new(ip_address, max_connection_timeout),
            None => anyhow::bail!(MissingIPAddressConfigurationError::new(guest_id)),
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
                        let difference = if snapshot.timestamp > disk_snapshot.timestamp {
                            snapshot.timestamp - disk_snapshot.timestamp
                        } else {
                            disk_snapshot.timestamp - snapshot.timestamp
                        };
                        difference.as_secs() < 300 // 5 minutes
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
            None => anyhow::bail!(UnknownNetworkError::new(network_id)),
        }
    }

    fn exists<T>(&self, path: T) -> bool
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();

        let test = self.prepare_host_command("test");
        let status = command_macros::command! {
            { test } -e (path)
        }
        .status();

        if let Ok(status) = status {
            status.success()
        } else {
            false
        }
    }

    fn forbid_remote_execution(&self) -> Result<()> {
        if self.host_connection.is_some() {
            anyhow::bail!(ForbiddenRemoteExecutionError::new());
        }

        Ok(())
    }

    fn is_booted<T>(&self, guest_id: T) -> Result<bool>
    where
        T: AsRef<str>,
    {
        let guest = self.get_guest(guest_id)?;

        if !self.exists(&guest.pidfile_path) || !self.exists(&guest.monitor_socket_path) {
            return Ok(false);
        }

        let pgrep = self.prepare_host_command("pgrep");
        let mut command = command_macros::command! {
            {pgrep} --full --pidfile (guest.pidfile_path) qemu
        };
        let status = command
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|_| ProcessExecutionError::new(&command))?;

        Ok(status.success())
    }

    fn create_parent_dir<T>(&self, path: T) -> Result<()>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();

        if let Some(parent_path) = path.parent() {
            let mkdir = self.prepare_host_command("mkdir");
            command_macros::command! {
                {mkdir} --mode 0755 -p (parent_path)
            }
            .execute()?;
        }

        Ok(())
    }

    fn prepare_host_command<T>(&self, command: T) -> Command
    where
        T: AsRef<str>,
    {
        let command = command.as_ref();

        if let Some(host_connection) = &self.host_connection {
            host_connection.prepare(command)
        } else {
            Command::new(command)
        }
    }
}
