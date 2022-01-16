use crate::command::Execute;
use crate::App;
use anyhow::Context;
use anyhow::Result;

impl App {
    pub fn initialize_guest<T>(&self, guest_id: T) -> Result<()>
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();

        let disks = self.get_guest_disks(guest_id)?;
        for disk in disks {
            let path = &disk.path;

            if path.exists() {
                continue;
            }

            if let Some(parent_path) = path.parent() {
                std::fs::create_dir_all(parent_path)
                    .with_context(|| format!("Failed to create parent folder for disk {path:?}"))?;
            }

            command_macros::command!(
                qemu-img create
                -q
                -fqcow2
                -olazy_refcounts=on
                -opreallocation=metadata
                (path)
                ((disk.size))G
            )
            .execute()?;

            command_macros::command!(
                qemu-img snapshot
                -croot
                (path)
            )
            .execute()?;
        }

        Ok(())
    }
}
