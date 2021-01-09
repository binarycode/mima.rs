use crate::App;
use anyhow::Result;
use std::io::Write;
use tabwriter::TabWriter;

impl App {
    pub fn list_guests(&self) -> Result<()> {
        let mut tw = TabWriter::new(std::io::stdout());
        writeln!(tw, "ID\tBOOTED\tSPICE\tDESCRIPTION")?;
        for (id, guest) in &self.guests {
            let booted = guest.is_booted()?;
            writeln!(
                tw,
                "{}\t{}\t{}\t{}",
                id, booted, guest.spice_port, guest.description
            )?;
        }
        tw.flush()?;

        Ok(())
    }
}
