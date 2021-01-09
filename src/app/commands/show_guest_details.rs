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

        let booted = guest.is_booted()?;
        writeln!(tw, "GUEST\tID\tBOOTED\tSPICE\tMEMORY\tCORES\tDESCRIPTION")?;
        writeln!(
            tw,
            "\t{}\t{}\t{}\t{}\t{}\t{}",
            guest_id, booted, guest.spice_port, guest.memory, guest.cores, guest.description,
        )?;
        writeln!(tw)?;
        tw.flush()?;

        writeln!(tw, "DISKS\tLABEL\tSIZE\tPATH")?;
        for disk in &guest.disks {
            writeln!(
                tw,
                "\t{}\t{}\t{}",
                disk.label,
                disk.size,
                disk.path.display()
            )?;
        }
        writeln!(tw)?;
        tw.flush()?;

        writeln!(tw, "NETWORK INTERFACES\tNETWORK\tMODEL\tMAC\tTAP")?;
        for network_interface in &guest.network_interfaces {
            writeln!(
                tw,
                "\t{}\t{}\t{}\t{}",
                network_interface.network_id,
                network_interface.model,
                network_interface.mac_address,
                network_interface.tap_name,
            )?;
        }
        tw.flush()?;

        Ok(())
    }
}
