use crate::command::Execute;
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

        if !guest.is_booted()? {
            return Ok(());
        }

        if !force {
            let monitor = command_macros::command!(
                socat
                -
                UNIX-CONNECT:(guest.monitor_socket_path)
            )
            .stdin(Stdio::piped())
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn()?;
            if let Some(mut stdin) = monitor.stdin {
                writeln!(stdin, "system_powerdown")?;
            }

            let delay = Duration::from_millis(1000);
            for _ in 0..wait {
                std::thread::sleep(delay);

                if !guest.is_booted()? {
                    return Ok(());
                }
            }
        }

        command_macros::command!(
            pkill
            --full
            --pidfile (guest.pidfile_path)
            qemu
        )
        .execute()?;

        Ok(())
    }
}
