use crate::App;
use anyhow::Result;
use std::io::Write;
use tabwriter::TabWriter;

impl App {
    pub fn show_guest_details<T>(&self, guest_id: T) -> Result<()>
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref();

        let guest = self.get_guest(guest_id)?;

        let mut tw = TabWriter::new(std::io::stdout());

        writeln!(tw, "GUEST\tID\tBOOTED\tSPICE\tMEMORY\tCORES\tDESCRIPTION").unwrap();
        writeln!(
            tw,
            "\t{guest_id}\t{booted}\t{spice_port}\t{memory}\t{cores}\t{description}",
            booted = self.is_booted(guest_id)?,
            cores = guest.cores,
            description = guest.description,
            memory = guest.memory,
            spice_port = guest.spice_port,
        )
        .unwrap();
        writeln!(tw).unwrap();
        tw.flush().unwrap();

        writeln!(tw, "DISKS\tLABEL\tSIZE\tPATH").unwrap();
        for disk in &guest.disks {
            writeln!(
                tw,
                "\t{label}\t{size}\t{path}",
                label = disk.label,
                path = disk.path.display(),
                size = disk.size,
            )
            .unwrap();
        }
        writeln!(tw).unwrap();
        tw.flush().unwrap();

        writeln!(tw, "NETWORK INTERFACES\tNETWORK\tMODEL\tMAC\tTAP").unwrap();
        for network_interface in &guest.network_interfaces {
            writeln!(
                tw,
                "\t{id}\t{model}\t{mac_address}\t{tap_name}",
                id = network_interface.network_id,
                mac_address = network_interface.mac_address,
                model = network_interface.model,
                tap_name = network_interface.tap_name,
            )
            .unwrap();
        }
        tw.flush().unwrap();

        Ok(())
    }
}
