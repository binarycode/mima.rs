use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub struct UnknownSnapshotError {
    guest_id: String,
    snapshot_id: String,
}

impl UnknownSnapshotError {
    pub fn new<T, U>(guest_id: T, snapshot_id: U) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let guest_id = guest_id.as_ref().to_string();
        let snapshot_id = snapshot_id.as_ref().to_string();

        Self {
            guest_id,
            snapshot_id,
        }
    }
}

impl Display for UnknownSnapshotError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let guest_id = self.guest_id.yellow();
        let snapshot_id = self.snapshot_id.to_string().yellow();

        write!(f, "unknown snapshot '{snapshot_id}' for guest '{guest_id}'")
    }
}

impl Error for UnknownSnapshotError {}
