use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

#[derive(Debug)]
pub struct UnknownGuestError {
    guest_id: String,
}

impl UnknownGuestError {
    pub fn new<T>(guest_id: T) -> Self
    where
        T: AsRef<str>,
    {
        let guest_id = guest_id.as_ref().to_string();

        Self { guest_id }
    }
}

impl Display for UnknownGuestError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let guest_id = self.guest_id.yellow();

        write!(f, "Unknown guest '{guest_id}'")
    }
}

impl Error for UnknownGuestError {}
