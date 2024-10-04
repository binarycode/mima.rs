use crate::app::BASH_COMMAND;
use crate::command::Execute;
use crate::errors::InvalidFileError;
use crate::errors::MissingInputPathOrStdinError;
use crate::App;
use anyhow::Result;
use std::io::IsTerminal;
use std::path::Path;
use std::process::Stdio;

impl App {
    pub fn execute_script_on_guest<T, U>(
        &self,
        guest_id: T,
        path: Option<U>,
        args: Vec<String>,
    ) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<Path>,
    {
        let connection = self.get_guest_ssh_connection(guest_id)?;

        let stdin: Stdio = if let Some(path) = &path {
            let path = path.as_ref();

            if !path.is_file() {
                anyhow::bail!(InvalidFileError::new(path));
            }

            std::fs::File::open(path)?.into()
        } else if !std::io::stdin().is_terminal() {
            Stdio::inherit()
        } else {
            anyhow::bail!(MissingInputPathOrStdinError::new());
        };

        let bash = connection.command(BASH_COMMAND);
        command_macros::command! {
            {bash} -s - [args]
        }
        .stdin(stdin)
        .stdout(Stdio::inherit())
        .execute()?;

        Ok(())
    }
}
