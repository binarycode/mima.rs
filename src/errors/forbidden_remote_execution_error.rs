use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub struct ForbiddenRemoteExecutionError {}

impl ForbiddenRemoteExecutionError {
    pub fn new() -> Self {
        Self {}
    }
}

impl Display for ForbiddenRemoteExecutionError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let host = "--host".yellow();
        write!(f, "argument '{host}' is not allowed for this command")
    }
}

impl Error for ForbiddenRemoteExecutionError {}
