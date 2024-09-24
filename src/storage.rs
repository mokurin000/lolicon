use std::{fs, path::Path};

use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

use lolicon::Result;

#[derive(Deserialize, Serialize)]
pub struct Storage {
    inner: FxHashSet<u64>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    /// If the set did not previously contain this value, true is returned.
    pub fn store(&mut self, pid: u64) -> bool {
        self.inner.insert(pid)
    }

    /// If the set contains this value, true is returned.
    pub fn contains(&self, pid: &u64) -> bool {
        self.inner.contains(pid)
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let json = fs::read_to_string(path)?;
        let data: Storage = serde_json::from_str(&json)?;
        Ok(data)
    }

    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = serde_json::to_string(&self)?;
        fs::write(path, &json)?;
        Ok(())
    }
}
