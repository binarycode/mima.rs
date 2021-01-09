use crate::command::Execute;
use crate::App;
use anyhow::Result;
use std::path::Path;

impl App {
    pub fn copy_file_to_guest<T, U>(&self, guest_id: T, path: U) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<Path>,
    {
        let path = path.as_ref();

        let guest = self.get_guest(guest_id)?;

        if !path.is_file() {
            anyhow::bail!("`{}` is not a file", path.display());
        }

        command_macros::command!(
            ssh
            -o ConnectionAttempts=3
            -o ConnectTimeout=60
            -o BatchMode=yes
            -o StrictHostKeyChecking=no
            -o UserKnownHostsFile=/dev/null
            root@(guest.ip_address)
            mkdir -p /root/mima
        )
        .execute()?;

        command_macros::command!(
            scp
            -o ConnectionAttempts=3
            -o ConnectTimeout=60
            -o BatchMode=yes
            -o StrictHostKeyChecking=no
            -o UserKnownHostsFile=/dev/null
            (path)
            root@(guest.ip_address):/root/mima/
        )
        .execute()?;

        Ok(())
    }
}
