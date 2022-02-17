use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::path::Path;

#[derive(Debug)]
pub struct ParseConfigurationError {
    path: String,
}

impl ParseConfigurationError {
    pub fn new<T>(path: T) -> Self
    where
        T: AsRef<Path>,
    {
        let path = path.as_ref().display().to_string();

        Self { path }
    }
}

impl Display for ParseConfigurationError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let path = self.path.yellow();

        write!(f, "Failed to parse configuration in '{path}'")
    }
}

impl Error for ParseConfigurationError {}
