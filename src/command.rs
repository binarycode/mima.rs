use anyhow::Context;
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
            .with_context(|| format!("Failed to run {:?}", self))?;

        let stdout = String::from_utf8(output.stdout).context("Failed to parse stdout")?;
        let stderr = String::from_utf8(output.stderr).context("Failed to parse stderr")?;

        if !output.status.success() {
            anyhow::bail!(indoc::formatdoc! {
                "
                    Failed to run {command:?}
                    stdout:
                    {stdout}
                    stderr:
                    {stderr}
                ",
                command = self,
                stdout = stdout,
                stderr = stderr,
            });
        }

        Ok(stdout)
    }

    fn execute_and_parse_json_output<T: DeserializeOwned>(&mut self) -> Result<T> {
        let stdout = self.execute()?;
        let value = serde_json::from_str(&stdout).with_context(|| {
            indoc::formatdoc! {
                "
                    Failed to parse output of {command:?}
                    stdout:
                    {stdout}
                ",
                command = self,
                stdout = stdout,
            }
        })?;

        Ok(value)
    }
}
