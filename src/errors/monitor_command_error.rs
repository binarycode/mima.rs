use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::path::Path;

#[derive(Debug)]
pub struct MonitorCommandError {
    command: String,
    monitor_socket_path: String,
}

impl MonitorCommandError {
    pub fn new<T, U>(monitor_socket_path: T, command: U) -> Self
    where
        T: AsRef<Path>,
        U: AsRef<str>,
    {
        let command = command.as_ref().to_string();
        let monitor_socket_path = monitor_socket_path.as_ref().display().to_string();

        Self {
            command,
            monitor_socket_path,
        }
    }
}

impl Display for MonitorCommandError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let command = self.command.yellow();
        let monitor_socket_path = self.monitor_socket_path.yellow();

        write!(
            f,
            "Failed to issue command '{command}' to monitor socket '{monitor_socket_path}'"
        )
    }
}

impl Error for MonitorCommandError {}
