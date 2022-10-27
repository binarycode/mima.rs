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
        for disk in disks {
            let path = &disk.path;

            if self.exists(&connection, path)? {
                continue;
            }

            self.create_parent_dir(&connection, path)?;

            let qemu_img = connection.command(QEMU_IMG_COMMAND);
            command_macros::command! {
                {qemu_img} create -q -fqcow2 -olazy_refcounts=on -opreallocation=metadata (path) ((disk.size))G
            }
            .execute()?;

            let qemu_img = connection.command(QEMU_IMG_COMMAND);
            command_macros::command! {
                {qemu_img} snapshot -croot (path)
            }
            .execute()?;
        }

        Ok(())
    }
}
