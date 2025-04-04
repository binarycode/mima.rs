use crate::command::Execute;
use crate::App;
use anyhow::Result;
use std::process::Stdio;

impl App {
    pub fn connect_to_guest<T>(&self, guest_id: T, args: Vec<String>) -> Result<()>
    where
        T: AsRef<str>,
    {
        let connection = self.get_guest_ssh_connection(guest_id)?;

        command_macros::command! {
            {connection.command()} [args]
        }
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .execute()?;

        Ok(())
    }
}
