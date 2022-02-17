use colored::*;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::path::Path;

#[derive(Debug)]
pub struct ParentFolderCreationError {
    parent_path: String,
    path: String,
}

impl ParentFolderCreationError {
    pub fn new<T, U>(path: T, parent_path: U) -> Self
    where
        T: AsRef<Path>,
        U: AsRef<Path>,
    {
        let parent_path = parent_path.as_ref().display().to_string();
        let path = path.as_ref().display().to_string();

        Self { parent_path, path }
    }
}

impl Display for ParentFolderCreationError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let parent_path = self.parent_path.yellow();
        let path = self.path.yellow();

        write!(
            f,
            "Failed to create parent folder '{parent_path}' for '{path}'"
        )
    }
}

impl Error for ParentFolderCreationError {}
