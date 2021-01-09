use crate::command::Execute;
use crate::App;
use anyhow::Result;
use std::path::PathBuf;

impl App {
    pub fn start_guest<T>(
        &self,
        guest_id: T,
        cdrom_path: Option<PathBuf>,
        floppy_path: Option<PathBuf>,
    ) -> Result<()>
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();

        let guest = self.get_guest(guest_id)?;

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
            if let Some(path) = cdrom_path {
                -boot d
                -device scsi-cd,drive=drive.cd0
                -drive "if"=none,id=drive.cd0,format=raw,media=cdrom,file=(path)
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

        Ok(())
    }
}
