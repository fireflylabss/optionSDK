use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Write bytes through a sibling temporary file and replace the destination.
///
/// The parent directory is created when needed. The temporary file is flushed
/// and synced before it is persisted, so callers never expose a partially
/// written settings or cache file to another process.
pub fn atomic_write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes.as_ref())?;
    temp.flush()?;
    temp.as_file().sync_all()?;
    temp.persist(path).map(|_| ()).map_err(|error| error.error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_replaces_destination() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("nested").join("state.toml");

        atomic_write(&path, b"first").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "first");

        atomic_write(&path, b"second").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "second");
    }
}
