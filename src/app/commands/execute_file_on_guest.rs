use crate::app::GUEST_WORKSPACE_PATH;
use crate::App;
use anyhow::Result;
use std::path::Path;

impl App {
    pub fn execute_file_on_guest<T, U>(
        &self,
        guest_id: T,
        path: U,
        max_connection_timeout: u64,
        args: Vec<String>,
    ) -> Result<()>
    where
        T: AsRef<str>,
        U: AsRef<Path>,
    {
        let path = path.as_ref();

        if !path.is_file() {
            anyhow::bail!("`{}` is not a file", path.display());
        }

        let guest_connection = self.get_guest_connection(guest_id, max_connection_timeout)?;
        guest_connection.execute(format!("mkdir -p {}", GUEST_WORKSPACE_PATH))?;
        guest_connection.upload(path, GUEST_WORKSPACE_PATH)?;

        let file_name = path.file_name().unwrap();
        let guest_path = Path::new(GUEST_WORKSPACE_PATH).join(file_name);
        guest_connection.execute(format!("chmod +x {}", guest_path.display()))?;
        guest_connection.execute_with_args(guest_path.to_string_lossy(), args)?;

        Ok(())
    }
}
