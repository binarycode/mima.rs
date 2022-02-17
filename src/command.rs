use crate::errors::CommandExecutionFailedError;
use crate::errors::ParseCommandOutputError;
use crate::errors::ParseStreamError;
use crate::errors::ProcessExecutionError;
use anyhow::Result;
use serde::de::DeserializeOwned;
use std::process::Command;

pub trait Execute {
    fn execute(&mut self) -> Result<String>;
    fn execute_and_parse_json_output<T: DeserializeOwned>(&mut self) -> Result<T>;
}

impl Execute for Command {
    fn execute(&mut self) -> Result<String> {
        let output = self
            .output()
            .map_err(|_| ProcessExecutionError::new(self))?;

        let stdout =
            String::from_utf8(output.stdout).map_err(|_| ParseStreamError::new("stdout"))?;
        let stderr =
            String::from_utf8(output.stderr).map_err(|_| ParseStreamError::new("stderr"))?;

        if !output.status.success() {
            anyhow::bail!(CommandExecutionFailedError::new(self, stdout, stderr));
        }

        Ok(stdout)
    }

    fn execute_and_parse_json_output<T: DeserializeOwned>(&mut self) -> Result<T> {
        let stdout = self.execute()?;
        let value = serde_json::from_str(&stdout)
            .map_err(|_| ParseCommandOutputError::new(self, stdout))?;

        Ok(value)
    }
}
