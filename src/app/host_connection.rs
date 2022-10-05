use crate::command::Execute;
use anyhow::Result;
use std::process::Command;
use std::time::Duration;

pub struct HostConnection {
    connection_timeout: u64,
    host: String,
}

impl HostConnection {
    pub fn new<T>(host: T) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let host = host.as_ref().to_owned();

        let mut connection_timeout = 1;
        loop {
            let result = command_macros::command! {
                ssh
                -o BatchMode=yes
                -o ConnectTimeout=((connection_timeout))
                -o StrictHostKeyChecking=no
                -o UserKnownHostsFile=/dev/null
                root@(host)
                exit 0
            }
            .execute();

            if result.is_ok() {
                return Ok(Self {
                    connection_timeout,
                    host,
                });
            }

            connection_timeout *= 2;

            if connection_timeout >= 60 {
                return Err(result.unwrap_err());
            } else {
                std::thread::sleep(Duration::from_secs(connection_timeout));
            }
        }
    }

    pub fn prepare<T>(&self, command: T) -> Command
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
            root@((self.host))
            [command]
        }
    }
}
