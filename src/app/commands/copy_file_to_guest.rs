use crate::app::MKDIR_COMMAND;
use crate::app::TEE_COMMAND;
use crate::command::Execute;
use crate::errors::InvalidFileError;
use crate::errors::MissingInputPathOrStdinError;
use crate::App;
use anyhow::Result;
use std::io::IsTerminal;
use std::path::Path;
use std::process::Stdio;

impl App {
    pub fn copy_file_to_guest<T, U, V>(
        &self,
        guest_id: T,
        target_path: U,
        source_path: Option<V>,
    ) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<Path>,
        V: AsRef<Path>,
    {
        let target_path = target_path.as_ref();

        let stdin: Stdio = if let Some(source_path) = &source_path {
            let source_path = source_path.as_ref();

            if !source_path.is_file() {
                anyhow::bail!(InvalidFileError::new(source_path));
            }

            std::fs::File::open(source_path)?.into()
        } else if !std::io::stdin().is_terminal() {
            Stdio::inherit()
        } else {
            anyhow::bail!(MissingInputPathOrStdinError::new());
        };

        let connection = self.get_guest_ssh_connection(&guest_id)?;

        let mkdir = connection.command(MKDIR_COMMAND);
        command_macros::command! {
            {mkdir} --parents "$(dirname" (target_path)")"
        }
        .execute()?;

        let tee = connection.command(TEE_COMMAND);
        command_macros::command! {
            {tee} (target_path)
        }
        .stdin(stdin)
        .execute()?;

        Ok(())
    }
}
