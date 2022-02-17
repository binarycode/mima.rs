use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub struct MissingIPAddressConfigurationError {
    guest_id: String,
}

impl MissingIPAddressConfigurationError {
    pub fn new<T>(guest_id: T) -> Self
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref().to_string();

        Self { guest_id }
    }
}

impl Display for MissingIPAddressConfigurationError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let guest_id = self.guest_id.yellow();

        write!(f, "IP address is not configured for guest '{guest_id}'")
    }
}

impl Error for MissingIPAddressConfigurationError {}
