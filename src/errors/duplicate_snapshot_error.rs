use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub struct DuplicateSnapshotError {
    disk_id: String,
    guest_id: String,
    snapshot_id: String,
}

impl DuplicateSnapshotError {
    pub fn new<T, U, V>(guest_id: T, disk_id: U, snapshot_id: V) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
        V: AsRef<str>,
    {
        let disk_id = disk_id.as_ref().to_string();
        let guest_id = guest_id.as_ref().to_string();
        let snapshot_id = snapshot_id.as_ref().to_string();

        Self {
            disk_id,
            guest_id,
            snapshot_id,
        }
    }
}

impl Display for DuplicateSnapshotError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let disk_id = self.disk_id.yellow();
        let guest_id = self.guest_id.yellow();
        let snapshot_id = self.snapshot_id.yellow();

        write!(
            f,
            "Disk '{disk_id}' of guest '{guest_id}' already contains snapshot '{snapshot_id}'"
        )
    }
}

impl Error for DuplicateSnapshotError {}
