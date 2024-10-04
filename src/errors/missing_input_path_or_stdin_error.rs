use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub struct MissingInputPathOrStdinError {}

impl MissingInputPathOrStdinError {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MissingInputPathOrStdinError {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for MissingInputPathOrStdinError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "missing input path or STDIN data")
    }
}

impl Error for MissingInputPathOrStdinError {}
