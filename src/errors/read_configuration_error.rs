use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::path::Path;

#[derive(Debug)]
pub struct ReadConfigurationError {
    path: String,
}

impl ReadConfigurationError {
    pub fn new<T>(path: T) -> Self
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref().display().to_string();

        Self { path }
    }
}

impl Display for ReadConfigurationError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let path = self.path.yellow();

        write!(f, "failed to read configuration from '{path}'")
    }
}

impl Error for ReadConfigurationError {}
