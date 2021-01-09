use crate::command::Execute;
use crate::App;
use anyhow::Result;

impl App {
    pub fn create_snapshot<T, U>(&self, guest_id: T, snapshot_id: U) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();
        let snapshot_id = snapshot_id.as_ref();

        let disks = self.get_guest_disks(guest_id)?;
        for (disk_id, disk) in disks.iter().enumerate() {
            let snapshots = self.get_disk_snapshots(guest_id, disk_id)?;
            if snapshots.contains_key(snapshot_id) {
                anyhow::bail!("Disk `{disk_label}` of guest `{guest_id}` already contains snapshot `{snapshot_id}`", disk_label = disk.label, guest_id = guest_id, snapshot_id = snapshot_id);
            }
        }

        for disk in disks {
            command_macros::command!(
                qemu-img snapshot
                -c(snapshot_id)
                (disk.path)
            )
            .execute()?;
        }

        Ok(())
    }
}
