use crate::App;
use anyhow::Result;
use std::path::Path;

impl App {
    pub fn execute_script_on_guest<T, U>(
        &self,
        guest_id: T,
        path: U,
        max_connection_timeout: u64,
    ) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<Path>,
    {
        let path = path.as_ref();

        if !path.is_file() {
            anyhow::bail!("{path:?} is not a file");
        }

        let file_name = path.file_name().unwrap().to_string_lossy();

        let guest_connection = self.get_guest_connection(guest_id, max_connection_timeout)?;
        guest_connection.execute("mkdir -p /root/mima")?;
        guest_connection.upload(path, "/root/mima")?;
        guest_connection.execute(format!("chmod +x /root/mima/{file_name}"))?;
        guest_connection.execute(format!("/root/mima/{file_name}"))?;

        Ok(())
    }
}
