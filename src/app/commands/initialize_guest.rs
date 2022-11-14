use crate::app::QEMU_IMG_COMMAND;
use crate::command::Execute;
use crate::App;
use anyhow::Result;

impl App {
    pub fn initialize_guest<T>(&self, guest_id: T) -> Result<()>
    where
        T: AsRef<str>,
    {
        let connection = self.get_host_ssh_connection();

        let guest_id = guest_id.as_ref();

        let disks = self.get_guest_disks(guest_id)?;
        let mut missing_disks = Vec::new();
        for disk in disks {
            if !self.exists(&connection, &disk.path)? {
                missing_disks.push(disk);
            }
        }
        let missing_disks = missing_disks;

        for disk in &missing_disks {
            let path = &disk.path;

            self.create_parent_dir(&connection, path)?;

            let qemu_img = connection.command(QEMU_IMG_COMMAND);
            command_macros::command! {
                {qemu_img} create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata (path) ((disk.size))G
            }
            .execute()?;
        }

        for disk in &missing_disks {
            let path = &disk.path;

            let qemu_img = connection.command(QEMU_IMG_COMMAND);
            command_macros::command! {
                {qemu_img} snapshot -croot (path)
            }
            .execute()?;
        }

        Ok(())
    }
}
