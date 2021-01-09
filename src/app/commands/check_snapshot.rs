use crate::App;
use anyhow::Result;

impl App {
    pub fn check_snapshot<T, U>(&self, guest_id: T, snapshot_id: U) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<str>,
    {
        let snapshot_id = snapshot_id.as_ref();

        let snapshots = self.get_guest_snapshots(guest_id)?;
        if !snapshots.contains_key(snapshot_id) {
            std::process::exit(1);
        }

        Ok(())
    }
}
