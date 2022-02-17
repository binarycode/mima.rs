use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub struct ParseStreamError {
    stream: String,
}

impl ParseStreamError {
    pub fn new<T>(stream: T) -> Self
    where
        T: AsRef<str>,
    {
        let stream = stream.as_ref().to_string();

        Self { stream }
    }
}

impl Display for ParseStreamError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let stream = self.stream.yellow();

        write!(f, "Failed to parse '{stream}' stream")
    }
}

impl Error for ParseStreamError {}
