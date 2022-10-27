use crate::App;
use anyhow::Result;
use std::io::Write;
use tabwriter::TabWriter;
use time::OffsetDateTime;

impl App {
    pub fn list_snapshots<T>(&self, guest_id: T) -> Result<()>
    where
        T: AsRef<str>,
    {
        let connection = self.get_host_ssh_connection();

        let mut snapshots: Vec<_> = self
            .get_guest_snapshots(&connection, guest_id)?
            .into_values()
            .collect();
        snapshots.sort_by_key(|snapshot| snapshot.timestamp);
        let snapshots = snapshots;

        let format =
            time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")?;
        let mut tw = TabWriter::new(std::io::stdout());
        writeln!(tw, "ID\tTIMESTAMP").unwrap();
        for snapshot in snapshots {
            let timestamp = OffsetDateTime::UNIX_EPOCH + snapshot.timestamp;
            writeln!(
                tw,
                "{id}\t{timestamp}",
                id = snapshot.id,
                timestamp = timestamp.format(&format)?,
            )
            .unwrap();
        }
        tw.flush().unwrap();

        Ok(())
    }
}
