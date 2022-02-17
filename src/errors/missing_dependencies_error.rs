use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub struct MissingDependenciesError {
    binaries: Vec<String>,
}

impl MissingDependenciesError {
    pub fn new<T, U>(binaries: T) -> Self
    where
        T: IntoIterator<Item = U>,
        U: AsRef<str>,
    {
        let mut binaries: Vec<String> = binaries
            .into_iter()
            .map(|binary| binary.as_ref().to_string())
            .collect();

        binaries.sort();

        Self { binaries }
    }
}

impl Display for MissingDependenciesError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let binaries = self
            .binaries
            .iter()
            .map(|binary| format!("'{}'", binary.yellow()))
            .collect::<Vec<String>>()
            .join(", ");

        write!(f, "Dependency missing: {binaries}")
    }
}

impl Error for MissingDependenciesError {}
