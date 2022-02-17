use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::process::Command;

#[derive(Debug)]
pub struct CommandExecutionFailedError {
    command: String,
    stderr: String,
    stdout: String,
}

impl CommandExecutionFailedError {
    pub fn new<T, U>(command: &Command, stdout: T, stderr: U) -> Self
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let command = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(" ");
        let stderr = stderr.as_ref().to_string();
        let stdout = stdout.as_ref().to_string();

        Self {
            command,
            stderr,
            stdout,
        }
    }
}

impl Display for CommandExecutionFailedError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        writeln!(f, "Failed to run '{}'", self.command.yellow())?;

        if !self.stdout.is_empty() {
            write!(f, "\nstdout:\n{}", self.stdout)?;
        }

        if !self.stderr.is_empty() {
            write!(f, "\nstderr:\n{}", self.stderr.red())?;
        }

        Ok(())
    }
}

impl Error for CommandExecutionFailedError {}
