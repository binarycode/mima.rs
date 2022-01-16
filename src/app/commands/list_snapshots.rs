use crate::App;
use anyhow::Result;
use std::io::Write;
use tabwriter::TabWriter;

impl App {
    pub fn list_snapshots<T>(&self, guest_id: T) -> Result<()>
    where
        T: AsRef<str>,
    {
        let mut snapshots: Vec<_> = self.get_guest_snapshots(guest_id)?.into_values().collect();
        snapshots.sort_by_key(|snapshot| snapshot.timestamp);
        let snapshots = snapshots;

        let mut tw = TabWriter::new(std::io::stdout());
        writeln!(tw, "ID\tTIMESTAMP")?;
        for snapshot in snapshots {
            writeln!(
                tw,
                "{id}\t{timestamp}",
                id = snapshot.id,
                timestamp = snapshot.timestamp.format("%Y-%m-%d %H:%M:%S"),
            )?;
        }
        tw.flush()?;

        Ok(())
    }
}
