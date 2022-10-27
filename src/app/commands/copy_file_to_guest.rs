use crate::app::GUEST_WORKSPACE_PATH;
use crate::app::MKDIR_COMMAND;
use crate::command::Execute;
use crate::errors::InvalidFileError;
use crate::App;
use anyhow::Result;
use std::path::Path;

impl App {
    pub fn copy_file_to_guest<T, U, V>(
        &self,
        guest_id: T,
        path: U,
        file_name: Option<V>,
    ) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<Path>,
        V: AsRef<str>,
    {
        let path = path.as_ref();

        if !path.is_file() {
            anyhow::bail!(InvalidFileError::new(path));
        }

        let connection = self.get_guest_ssh_connection(&guest_id)?;

        let mkdir = connection.command(MKDIR_COMMAND);
        command_macros::command! {
            {mkdir} -p (GUEST_WORKSPACE_PATH)
        }
        .execute()?;

        let mut destination_path = GUEST_WORKSPACE_PATH.to_owned();
        if let Some(file_name) = file_name {
            let file_name = file_name.as_ref();
            destination_path = Path::new(&destination_path)
                .join(file_name)
                .to_string_lossy()
                .into_owned()
        }
        let destination_path = destination_path;

        connection.upload(path, destination_path)?;

        Ok(())
    }
}
