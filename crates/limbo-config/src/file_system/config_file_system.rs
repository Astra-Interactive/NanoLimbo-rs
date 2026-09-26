use std::io;
use std::path::Path;

/// The three things configuration loading needs from a filesystem.
///
/// Reading `settings.yml` is the only impure part of this crate, and `@file` references
/// mean the impurity reaches into the middle of parsing. Behind a port, the whole of it —
/// including which file a credential was read from — is testable without a temporary
/// directory, and the composition root stays the only place that touches the disk.
pub trait ConfigFileSystem {
    fn exists(&self, path: &Path) -> bool;

    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;

    /// Creates `path` with `contents`. Only ever called for a file that is not there.
    fn write(&self, path: &Path, contents: &[u8]) -> io::Result<()>;
}
