use std::fs;
use std::io;
use std::path::Path;

use crate::config_file_system::ConfigFileSystem;

/// The real filesystem. Created by the composition root and handed to the loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskFileSystem;

impl ConfigFileSystem for DiskFileSystem {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn write(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
        fs::write(path, contents)
    }
}
