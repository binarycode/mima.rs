use crate::command::Execute;
use anyhow::Result;
use std::path::Path;
use std::time::Duration;

pub struct GuestConnection {
    connection_timeout: u64,
    ip_address: String,
}

impl GuestConnection {
    pub fn new<T>(ip_address: T, max_connection_timeout: u64) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let ip_address = ip_address.as_ref().to_owned();

        let mut connection_timeout = 1;
        loop {
            let result = command_macros::command! {
                ssh
                -o BatchMode=yes
                -o ConnectTimeout=((connection_timeout))
                -o StrictHostKeyChecking=no
                -o UserKnownHostsFile=/dev/null
                root@(ip_address)
                exit 0
            }
            .execute();

            if result.is_ok() {
                return Ok(Self {
                    connection_timeout,
                    ip_address,
                });
            }

            connection_timeout *= 2;

            if connection_timeout >= max_connection_timeout {
                return Err(result.unwrap_err());
            } else {
                std::thread::sleep(Duration::from_secs(connection_timeout));
            }
        }
    }

    pub fn execute<T>(&self, command: T) -> Result<()>
    where
        T: AsRef<str>,
    {
        self.execute_with_args(command, Vec::new())
    }

    pub fn execute_with_args<T>(&self, command: T, args: Vec<String>) -> Result<()>
    where
        T: AsRef<str>,
    {
        let command = command.as_ref().split_whitespace();

        command_macros::command! {
            ssh
            -o BatchMode=yes
            -o ConnectTimeout=((self.connection_timeout))
            -o StrictHostKeyChecking=no
            -o UserKnownHostsFile=/dev/null
            -A
            root@(self.ip_address)
            [command]
            [args]
        }
        .execute()?;

        Ok(())
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
            root@(self.ip_address):(destination_path)
        }
        .execute()?;

        Ok(())
    }
}
