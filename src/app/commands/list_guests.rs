use crate::App;
use anyhow::Result;
use std::io::Write;
use tabwriter::TabWriter;

impl App {
    pub fn list_guests(&self) -> Result<()> {
        let mut tw = TabWriter::new(std::io::stdout());
        writeln!(tw, "ID\tBOOTED\tSPICE\tDESCRIPTION")?;
        for (id, guest) in &self.guests {
            writeln!(
                tw,
                "{id}\t{booted}\t{spice_port}\t{description}",
                booted = guest.is_booted()?,
                description = guest.description,
                spice_port = guest.spice_port,
            )?;
        }
        tw.flush()?;

        Ok(())
    }
}
