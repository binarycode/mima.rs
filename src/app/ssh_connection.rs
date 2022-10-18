use super::PROBE_COMMAND;
use crate::command::Execute;
use anyhow::Result;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const ROOT_USER: &str = "root";

pub struct SshConnection {
    connection_timeout: u64,
    destination: String,
}

impl SshConnection {
    pub fn new<T>(destination: T, max_connection_timeout: u64) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let mut app = Self {
            connection_timeout: 1,
            destination: destination.as_ref().to_owned(),
        };

        loop {
            let result = app.command(PROBE_COMMAND).execute();

            if result.is_ok() {
                return Ok(app);
            }

            app.connection_timeout *= 2;

            if app.connection_timeout >= max_connection_timeout {
                return Err(result.unwrap_err());
            } else {
                std::thread::sleep(Duration::from_secs(app.connection_timeout));
            }
        }
    }

    pub fn command<T>(&self, command: T) -> Command
    where
        T: AsRef<str>,
    {
        let command = command.as_ref();

        command_macros::command! {
            ssh
            -o BatchMode=yes
            -o ConnectTimeout=((self.connection_timeout))
            -o ForwardAgent=yes
            -o StrictHostKeyChecking=no
            -o UserKnownHostsFile=/dev/null
            (ROOT_USER)@(self.destination)
            (command)
        }
    }

    pub fn upload<T, U>(&self, source_path: T, destination_path: U) -> Result<()>
    where
        T: AsRef<Path>,
        U: AsRef<Path>,
    {
        let source_path = source_path.as_ref();
        let destination_path = destination_path.as_ref();

        command_macros::command! {
            scp
            -o BatchMode=yes
            -o ConnectTimeout=((self.connection_timeout))
            -o StrictHostKeyChecking=no
            -o UserKnownHostsFile=/dev/null
            (source_path)
            root@(self.destination):(destination_path)
        }
        .execute()?;

        Ok(())
    }
}
