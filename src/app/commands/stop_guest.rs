use crate::app::PKILL_COMMAND;
use crate::app::SOCAT_COMMAND;
use crate::command::Execute;
use crate::errors::MonitorCommandError;
use crate::errors::ProcessExecutionError;
use crate::App;
use anyhow::Result;
use std::io::Write;
use std::process::Stdio;
use std::time::Duration;

impl App {
    pub fn stop_guest<T>(&self, guest_id: T, wait: u64, force: bool) -> Result<()>
    where
        T: AsRef<str>,
    {
        let connection = self.get_host_ssh_connection();

        let guest_id = guest_id.as_ref();

        let guest = self.get_guest(&guest_id)?;

        if !self.is_booted(&connection, guest_id)? {
            return Ok(());
        }

        if !force {
            let socat = connection.command(SOCAT_COMMAND);
            let mut command = command_macros::command! {
                {socat} - UNIX-CONNECT:(guest.monitor_socket_path)
            };
            let monitor = command
                .stdin(Stdio::piped())
                .stderr(Stdio::null())
                .stdout(Stdio::null())
                .spawn()
                .map_err(|_| ProcessExecutionError::new(&command))?;
            if let Some(mut stdin) = monitor.stdin {
                writeln!(stdin, "system_powerdown").map_err(|_| {
                    MonitorCommandError::new(&guest.monitor_socket_path, "system_powerdown")
                })?;
            }

            let delay = Duration::from_millis(1000);
            for _ in 0..wait {
                std::thread::sleep(delay);

                if !self.is_booted(&connection, guest_id)? {
                    return Ok(());
                }
            }
        }

        let pkill = connection.command(PKILL_COMMAND);
        command_macros::command! {
            {pkill} --full --pidfile (guest.pidfile_path) qemu
        }
        .execute()?;

        Ok(())
    }
}
