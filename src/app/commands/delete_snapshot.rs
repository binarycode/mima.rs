use crate::app::QEMU_IMG_COMMAND;
use crate::command::Execute;
use crate::App;
use anyhow::Result;

impl App {
    pub fn delete_snapshot<T, U>(&self, guest_id: T, snapshot_id: U) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let connection = self.get_host_ssh_connection();

        let guest_id = guest_id.as_ref();
        let snapshot_id = snapshot_id.as_ref();

        let disks = self.get_guest_disks(guest_id)?;
        for disk in disks {
            command_macros::command! {
                {connection.execute(QEMU_IMG_COMMAND)} snapshot -d(snapshot_id) (disk.path)
            }
            .execute()?;
        }

        Ok(())
    }
}
