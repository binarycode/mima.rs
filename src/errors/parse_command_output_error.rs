use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::process::Command;

#[derive(Debug)]
pub struct ParseCommandOutputError {
    command: String,
    stdout: String,
}

impl ParseCommandOutputError {
    pub fn new<T>(command: &Command, stdout: T) -> Self
    where
        T: AsRef<str>,
    {
        let command = std::iter::once(command.get_program())
            .chain(command.get_args())
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(" ");
        let stdout = stdout.as_ref().to_string();

        Self { command, stdout }
    }
}

impl Display for ParseCommandOutputError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        writeln!(f, "Failed to parse output of '{}'", self.command.yellow())?;

        if !self.stdout.is_empty() {
            write!(f, "\nstdout:\n{}", self.stdout)?;
        }

        Ok(())
    }
}

impl Error for ParseCommandOutputError {}
