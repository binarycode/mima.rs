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
        let guest_id = guest_id.as_ref();

        let guest = self.get_guest(&guest_id)?;

        if !self.is_booted(guest_id)? {
            return Ok(());
        }

        if !force {
            let socat = self.prepare_host_command("socat");
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

                if !self.is_booted(guest_id)? {
                    return Ok(());
                }
            }
        }

        let pkill = self.prepare_host_command("pkill");
        command_macros::command! {
            {pkill} --full --pidfile (guest.pidfile_path) qemu
        }
        .execute()?;

        Ok(())
    }
}
