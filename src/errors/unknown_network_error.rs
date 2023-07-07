use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub struct UnknownNetworkError {
    network_id: String,
}

impl UnknownNetworkError {
    pub fn new<T>(network_id: T) -> Self
    where
        T: AsRef<str>,
    {
        let network_id = network_id.as_ref().to_string();

        Self { network_id }
    }
}

impl Display for UnknownNetworkError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let network_id = self.network_id.yellow();

        write!(f, "unknown network '{network_id}'")
    }
}

impl Error for UnknownNetworkError {}
