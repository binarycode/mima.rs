use crate::app::BASH_COMMAND;
use crate::app::CHMOD_COMMAND;
use crate::app::GUEST_WORKSPACE_PATH;
use crate::app::MKDIR_COMMAND;
use crate::app::RM_COMMAND;
use crate::command::Execute;
use crate::errors::InvalidFileError;
use crate::App;
use anyhow::Result;
use std::io::IsTerminal;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;

impl App {
    pub fn execute_script_on_guest<T>(
        &self,
        guest_id: T,
        path: Option<PathBuf>,
        args: Vec<String>,
    ) -> Result<()>
    where
        T: AsRef<str>,
    {
        let connection = self.get_guest_ssh_connection(guest_id)?;

        if let Some(path) = &path {
            if !path.is_file() {
                anyhow::bail!(InvalidFileError::new(path));
            }

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
            .stdout(Stdio::inherit())
            .execute()?;

            let rm = connection.command(RM_COMMAND);
            command_macros::command! {
                {rm} -rf (guest_path)
            }
            .execute()?;
        } else if !std::io::stdin().is_terminal() {
            connection
                .command(BASH_COMMAND)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .execute()?;
        }

        Ok(())
    }
}
