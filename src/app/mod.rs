mod commands;
mod disk;
mod guest;
mod network;
mod network_interface;
mod snapshot;
mod ssh_connection;

use crate::command::Execute;
use crate::errors::ParseConfigurationError;
use crate::errors::ProcessExecutionError;
use crate::errors::ReadConfigurationError;
use crate::errors::UnknownGuestError;
use crate::errors::UnknownNetworkError;
use anyhow::Result;
use disk::Disk;
use guest::Guest;
use network::Network;
use serde::Deserialize;
use snapshot::Snapshot;
use ssh_connection::SshConnection;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

const GUEST_WORKSPACE_PATH: &str = "/tmp";

const BASH_COMMAND: &str = "bash";
const CHMOD_COMMAND: &str = "chmod";
const IP_COMMAND: &str = "ip";
const MKDIR_COMMAND: &str = "mkdir";
const PGREP_COMMMAND: &str = "pgrep";
const PKILL_COMMAND: &str = "pkill";
const RM_COMMAND: &str = "rm";
const SOCAT_COMMAND: &str = "socat";
const TEST_COMMAND: &str = "test";
const QEMU_COMMAND: &str = "qemu-system-x86_64";
const QEMU_IMG_COMMAND: &str = "qemu-img";

#[derive(Deserialize)]
pub struct App {
    host: String,
    guests: BTreeMap<String, Guest>,
    networks: BTreeMap<String, Network>,
}

impl App {
    pub fn new<T>(path: T) -> Result<Self>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();

        let config =
            std::fs::read_to_string(path).map_err(|_| ReadConfigurationError::new(path))?;

        let app =
            toml::from_str::<Self>(&config).map_err(|_| ParseConfigurationError::new(path))?;

        Ok(app)
    }

    fn get_disk_snapshots<T>(
        &self,
        connection: &SshConnection,
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

        let qemu_img = connection.command(QEMU_IMG_COMMAND);
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

    fn get_guest<T>(&self, guest_id: T) -> Result<&Guest>
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();

        match self.guests.get(guest_id) {
            Some(guest) => Ok(guest),
            None => anyhow::bail!(UnknownGuestError::new(guest_id)),
        }
    }

    fn get_guest_disks<T>(&self, guest_id: T) -> Result<&Vec<Disk>>
    where
        T: AsRef<str>,
    {
        let guest = self.get_guest(guest_id)?;

        Ok(&guest.disks)
    }

    fn get_guest_snapshots<T>(
        &self,
        connection: &SshConnection,
        guest_id: T,
    ) -> Result<HashMap<String, Snapshot>>
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();

        let mut first_disk = true;
        let mut snapshots = HashMap::new();

        let disks = self.get_guest_disks(guest_id)?;
        for disk_id in 0..disks.len() {
            let disk_snapshots = self.get_disk_snapshots(connection, guest_id, disk_id)?;

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

    fn get_guest_ssh_connection<T>(&self, guest_id: T) -> Result<SshConnection>
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();

        let guest = self.get_guest(guest_id)?;

        Ok(SshConnection::new(&guest.ip_address))
    }

    fn get_host_ssh_connection(&self) -> SshConnection {
        SshConnection::new(&self.host)
    }

    fn get_network<T>(&self, network_id: T) -> Result<&Network>
    where
        T: AsRef<str>,
    {
        let network_id = network_id.as_ref();

        match self.networks.get(network_id) {
            Some(network) => Ok(network),
            None => anyhow::bail!(UnknownNetworkError::new(network_id)),
        }
    }

    fn exists<T>(&self, connection: &SshConnection, path: T) -> Result<bool>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();

        let test = connection.command(TEST_COMMAND);
        let status = command_macros::command! {
            {test} -e (path)
        }
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

        if let Ok(status) = status {
            Ok(status.success())
        } else {
            Ok(false)
        }
    }

    fn is_booted<T>(&self, connection: &SshConnection, guest_id: T) -> Result<bool>
    where
        T: AsRef<str>,
    {
        let guest = self.get_guest(guest_id)?;

        let pgrep = connection.command(PGREP_COMMMAND);
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

    fn create_parent_dir<T>(&self, connection: &SshConnection, path: T) -> Result<()>
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref();

        if let Some(parent_path) = path.parent() {
            let mkdir = connection.command(MKDIR_COMMAND);
            command_macros::command! {
                {mkdir} --mode 0755 -p (parent_path)
            }
            .execute()?;
        }

        Ok(())
    }
}
