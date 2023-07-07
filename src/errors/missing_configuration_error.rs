use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::path::Path;

#[derive(Debug)]
pub struct MissingConfigurationError {
    paths: Vec<String>,
}

impl MissingConfigurationError {
    pub fn new<T>(paths: &[T]) -> Self
    where
        T: AsRef<Path>,
    {
        let paths = paths
            .iter()
            .map(|path| path.as_ref().display().to_string())
            .collect();

        Self { paths }
    }
}

impl Display for MissingConfigurationError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let paths = self
            .paths
            .iter()
            .map(|path| format!("'{path}'", path = path.yellow()))
            .collect::<Vec<String>>()
            .join(", ");

        write!(f, "missing configuration at {paths}")
    }
}

impl Error for MissingConfigurationError {}
