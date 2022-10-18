use crate::app::QEMU_IMG_COMMAND;
use crate::command::Execute;
use crate::errors::UnknownSnapshotError;
use crate::App;
use anyhow::Result;

impl App {
    pub fn apply_snapshot<T, U>(&self, guest_id: T, snapshot_id: U) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();
        let snapshot_id = snapshot_id.as_ref();

        let snapshots = self.get_guest_snapshots(guest_id)?;
        if !snapshots.contains_key(snapshot_id) {
            anyhow::bail!(UnknownSnapshotError::new(guest_id, snapshot_id));
        }

        let connection = self.get_host_ssh_connection()?;

        let disks = self.get_guest_disks(guest_id)?;
        for disk in disks {
            let qemu_img = connection.command(QEMU_IMG_COMMAND);
            command_macros::command! {
                {qemu_img} snapshot -a(snapshot_id) (disk.path)
            }
            .execute()?;
        }

        Ok(())
    }
}
