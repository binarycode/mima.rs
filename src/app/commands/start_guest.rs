use crate::command::Execute;
use crate::errors::ParentFolderCreationError;
use crate::errors::SetPermissionsError;
use crate::App;
use anyhow::Result;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

impl App {
    pub fn start_guest<T>(
        &self,
        guest_id: T,
        boot_from_cdrom: bool,
        boot_from_network: bool,
        cdrom_paths: Vec<PathBuf>,
        floppy_path: Option<PathBuf>,
    ) -> Result<()>
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();

        let guest = self.get_guest(guest_id)?;

        if guest.is_booted()? {
            return Ok(());
        }

        create_parent_dir(&guest.monitor_socket_path)?;
        create_parent_dir(&guest.pidfile_path)?;

        command_macros::command!(
            qemu-system-x86_64
            -name (guest_id)
            -machine q35,accel=kvm
            -cpu host
            -m ((guest.memory))M
            -smp ((guest.cores))
            -no-user-config
            -nodefaults
            -daemonize
            -runas nobody
            -monitor unix:(guest.monitor_socket_path),server,nowait
            -pidfile (guest.pidfile_path)
            -vga std
            -spice port=((guest.spice_port)),disable-ticketing=on
            -object iothread,id=iothread1
            -device virtio-scsi-pci-non-transitional,iothread=iothread1
            for network_interface in &guest.network_interfaces {
                -device (network_interface.model),netdev=network.(network_interface.tap_name),mac=(network_interface.mac_address)
                -netdev tap,id=network.(network_interface.tap_name),ifname=(network_interface.tap_name),script=no,downscript=no
            }
            for disk in &guest.disks {
                -device scsi-hd,drive=drive.(disk.label)
                -drive "if"=none,id=drive.(disk.label),format=qcow2,file=(disk.path)
            }
            if boot_from_cdrom {
                -boot d
            }
            if boot_from_network {
                -boot n
            }
            for (i, path) in cdrom_paths.iter().enumerate() {
                -device scsi-cd,drive=drive.cd((i))
                -drive "if"=none,id=drive.cd((i)),format=raw,media=cdrom,file=(path)
            }
            if let Some(path) = floppy_path {
                -drive "if"=floppy,id=drive.fd0,format=raw,file=fat:floppy:rw:(path)
            }
        )
        .execute()?;

        for network_interface in &guest.network_interfaces {
            let network = self.get_network(&network_interface.network_id)?;
            command_macros::command!(
                ip link set (network_interface.tap_name) master (network.bridge_name) up
            )
            .execute()?;
        }

        if guest.pidfile_path.exists() {
            let permissions = Permissions::from_mode(0o644);
            std::fs::set_permissions(&guest.pidfile_path, permissions.clone())
                .map_err(|_| SetPermissionsError::new(&guest.pidfile_path, permissions))?;
        }

        Ok(())
    }
}

fn create_parent_dir<T>(path: T) -> Result<()>
where
    T: AsRef<Path>,
{
    let path = path.as_ref();

    if let Some(parent_path) = path.parent() {
        if !parent_path.exists() {
            std::fs::create_dir_all(parent_path)
                .map_err(|_| ParentFolderCreationError::new(path, parent_path))?;

            let permissions = Permissions::from_mode(0o755);
            std::fs::set_permissions(parent_path, permissions.clone())
                .map_err(|_| SetPermissionsError::new(parent_path, permissions))?;
        }
    }

    Ok(())
}
