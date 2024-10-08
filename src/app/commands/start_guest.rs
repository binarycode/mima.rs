use crate::app::CHMOD_COMMAND;
use crate::app::IP_COMMAND;
use crate::app::QEMU_COMMAND;
use crate::command::Execute;
use crate::App;
use anyhow::Result;
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
        let connection = self.get_host_ssh_connection();

        let guest_id = guest_id.as_ref();

        let guest = self.get_guest(guest_id)?;

        if self.is_booted(&connection, guest_id)? {
            return Ok(());
        }

        self.create_parent_dir(&connection, &guest.monitor_socket_path)?;
        self.create_parent_dir(&connection, &guest.pidfile_path)?;

        command_macros::command! {
            {connection.execute(QEMU_COMMAND)}
            -name (guest_id)
            -machine q35,accel=kvm
            -cpu (guest.host)
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
            for (i, disk) in guest.disks.iter().enumerate() {
                -device (disk.model),drive=drive.(disk.label)
                if disk.model == "scsi-hd" {
                    -drive "if"=none,id=drive.(disk.label),format=qcow2,file=(disk.path),index=((i))
                } else {
                    -drive "if"=none,id=drive.(disk.label),format=qcow2,file=(disk.path)
                }
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
        }
        .execute()?;

        for network_interface in &guest.network_interfaces {
            let network = self.get_network(&network_interface.network_id)?;
            command_macros::command! {
                {connection.execute(IP_COMMAND)} link set (network_interface.tap_name) master (network.bridge_name) up
            }
            .execute()?;
        }

        command_macros::command! {
            {connection.execute(CHMOD_COMMAND)} 644 (guest.pidfile_path)
        }
        .execute()?;

        Ok(())
    }
}
