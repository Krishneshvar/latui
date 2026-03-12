use std::path::PathBuf;

/// SQLite database for usage tracking
pub struct Database {
    // TODO: Add rusqlite Connection when dependency is added
}

impl Database {
    pub fn new(_path: PathBuf) -> Result<Self, String> {
        // TODO: Initialize SQLite database
        Ok(Self {})
    }

    pub fn init_schema(&self) -> Result<(), String> {
        // TODO: Create tables
        Ok(())
    }
}
