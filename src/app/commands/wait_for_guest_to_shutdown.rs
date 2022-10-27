use crate::App;
use anyhow::Result;
use std::time::Duration;

impl App {
    pub fn wait_for_guest_to_shutdown<T>(&self, guest_id: T) -> Result<()>
    where
        T: AsRef<str>,
    {
        let connection = self.get_host_ssh_connection();

        let delay = Duration::from_millis(1000);
        while self.is_booted(&connection, &guest_id)? {
            std::thread::sleep(delay);
        }

        Ok(())
    }
}
