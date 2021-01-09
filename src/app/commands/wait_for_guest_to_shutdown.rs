use crate::App;
use anyhow::Result;
use std::time::Duration;

impl App {
    pub fn wait_for_guest_to_shutdown<T>(&self, guest_id: T) -> Result<()>
    where
        T: AsRef<str>,
    {
        let guest = self.get_guest(guest_id)?;

        let delay = Duration::from_millis(1000);
        while guest.is_booted()? {
            std::thread::sleep(delay);
        }

        Ok(())
    }
}
