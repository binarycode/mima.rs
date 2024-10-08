use crate::app::QEMU_IMG_COMMAND;
use crate::command::Execute;
use crate::errors::DuplicateSnapshotError;
use crate::App;
use anyhow::Result;

impl App {
    pub fn create_snapshot<T, U>(&self, guest_id: T, snapshot_id: U) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let connection = self.get_host_ssh_connection();

        let guest_id = guest_id.as_ref();
        let snapshot_id = snapshot_id.as_ref();

        let disks = self.get_guest_disks(guest_id)?;
        for (disk_id, disk) in disks.iter().enumerate() {
            let snapshots = self.get_disk_snapshots(&connection, guest_id, disk_id)?;
            if snapshots.contains_key(snapshot_id) {
                anyhow::bail!(DuplicateSnapshotError::new(
                    guest_id,
                    &disk.label,
                    snapshot_id
                ));
            }
        }

        for disk in disks {
            command_macros::command! {
                {connection.execute(QEMU_IMG_COMMAND)} snapshot -c(snapshot_id) (disk.path)
            }
            .execute()?;
        }

        Ok(())
    }
}
