use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::process::Command;

#[derive(Debug)]
pub struct ProcessExecutionError {
    command: String,
}

impl ProcessExecutionError {
    pub fn new(command: &Command) -> Self {
        let command = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(" ");

        Self { command }
    }
}

impl Display for ProcessExecutionError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Failed to run '{}'", self.command.yellow())
    }
}

impl Error for ProcessExecutionError {}
