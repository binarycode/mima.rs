use crate::app::GUEST_WORKSPACE_PATH;
use crate::errors::InvalidFileError;
use crate::App;
use anyhow::Result;
use std::path::Path;

impl App {
    pub fn copy_file_to_guest<T, U>(
        &self,
        guest_id: T,
        path: U,
        max_connection_timeout: u64,
    ) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<Path>,
    {
        self.forbid_remote_execution()?;

        let path = path.as_ref();

        if !path.is_file() {
            anyhow::bail!(InvalidFileError::new(path));
        }

        let guest_connection = self.get_guest_connection(guest_id, max_connection_timeout)?;
        guest_connection.execute(format!("mkdir -p {GUEST_WORKSPACE_PATH}"))?;
        guest_connection.upload(path, GUEST_WORKSPACE_PATH)?;

        Ok(())
    }
}
