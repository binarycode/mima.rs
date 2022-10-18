use crate::app::CHMOD_COMMAND;
use crate::app::GUEST_WORKSPACE_PATH;
use crate::app::MKDIR_COMMAND;
use crate::command::Execute;
use crate::errors::InvalidFileError;
use crate::App;
use anyhow::Result;
use std::path::Path;

impl App {
    pub fn execute_file_on_guest<T, U>(
        &self,
        guest_id: T,
        path: U,
        timeout: u64,
        args: Vec<String>,
    ) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<Path>,
    {
        let path = path.as_ref();

        if !path.is_file() {
            anyhow::bail!(InvalidFileError::new(path));
        }

        let connection = self.get_guest_ssh_connection(guest_id, timeout)?;

        let mkdir = connection.command(MKDIR_COMMAND);
        command_macros::command! {
            {mkdir} -p (GUEST_WORKSPACE_PATH)
        }
        .execute()?;

        connection.upload(path, GUEST_WORKSPACE_PATH)?;

        let file_name = path.file_name().unwrap();
        let guest_path = Path::new(GUEST_WORKSPACE_PATH).join(file_name);

        let chmod = connection.command(CHMOD_COMMAND);
        command_macros::command! {
            {chmod} +x (guest_path)
        }
        .execute()?;

        let file = connection.command(guest_path.display().to_string());
        command_macros::command! {
            {file} [args]
        }
        .execute()?;

        Ok(())
    }
}
