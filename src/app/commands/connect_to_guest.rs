use crate::command::Execute;
use crate::App;
use anyhow::Result;
use std::io::IsTerminal;
use std::process::Stdio;

impl App {
    pub fn connect_to_guest<T>(&self, guest_id: T, args: Vec<String>) -> Result<()>
    where
        T: AsRef<str>,
    {
        let connection = self.get_guest_ssh_connection(guest_id)?;

        let stdin: Stdio = if std::io::stdin().is_terminal() {
            Stdio::null()
        } else {
            Stdio::inherit()
        };

        command_macros::command! {
            {connection.command()} [args]
        }
        .stdin(stdin)
        .stdout(Stdio::inherit())
        .execute()?;

        Ok(())
    }
}
