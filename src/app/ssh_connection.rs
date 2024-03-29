use crate::command::Execute;
use anyhow::Result;
use std::path::Path;
use std::process::Command;

const ROOT_USER: &str = "root";
const CONNECTION_TIMEOUT: u64 = 10;

pub struct SshConnection {
    destination: String,
}

impl SshConnection {
    pub fn new<T>(destination: T) -> Self
    where
        T: AsRef<str>,
    {
        Self {
            destination: destination.as_ref().to_owned(),
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
            -o ConnectTimeout=((CONNECTION_TIMEOUT))
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
            -o ConnectTimeout=((CONNECTION_TIMEOUT))
            -o StrictHostKeyChecking=no
            -o UserKnownHostsFile=/dev/null
            (source_path)
            root@(self.destination):(destination_path)
        }
        .execute()?;

        Ok(())
    }
}
