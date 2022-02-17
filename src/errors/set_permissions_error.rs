use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Debug)]
pub struct SetPermissionsError {
    path: String,
    permissions: Permissions,
}

impl SetPermissionsError {
    pub fn new<T>(path: T, permissions: Permissions) -> Self
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref().display().to_string();

        Self { path, permissions }
    }
}

impl Display for SetPermissionsError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let path = self.path.yellow();
        let permissions = format!("{:o}", self.permissions.mode());

        write!(f, "Failed to set permissions '{permissions}' on '{path}'")
    }
}

impl Error for SetPermissionsError {}
