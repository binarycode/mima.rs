use crate::app::GUEST_WORKSPACE_PATH;
use crate::app::MKDIR_COMMAND;
use crate::command::Execute;
use crate::errors::InvalidFileError;
use crate::App;
use anyhow::Result;
use std::path::Path;

impl App {
    pub fn copy_file_to_guest<T, U>(&self, guest_id: T, path: U, timeout: u64) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<Path>,
    {
        let path = path.as_ref();

        if !path.is_file() {
            anyhow::bail!(InvalidFileError::new(path));
        }

        let connection = self.get_guest_ssh_connection(&guest_id, timeout)?;

        let mkdir = connection.command(MKDIR_COMMAND);
        command_macros::command! {
            {mkdir} -p (GUEST_WORKSPACE_PATH)
        }
        .execute()?;

        connection.upload(path, GUEST_WORKSPACE_PATH)?;

        Ok(())
    }
}
