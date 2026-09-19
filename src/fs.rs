use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Write bytes through a sibling temporary file and replace the destination.
///
/// The parent directory is created when needed. The temporary file is flushed
/// and synced before it is persisted, so callers never expose a partially
/// written settings or cache file to another process.
///
/// Contract:
///
/// - On create, the destination keeps the temporary file's default mode
///   (`0600` from `tempfile`); no chmod is applied. This is the correct
///   default for key material.
/// - On replace, the previous destination's permission bits are re-applied
///   afterwards (Unix only; other platforms keep the remapped defaults).
///   The pre-write metadata read may race with concurrent replacers; that
///   TOCTOU is benign here because the worst case is preserving slightly
///   stale permission bits, never half-written content.
/// - After the rename, the parent directory is fsynced so a crash cannot lose
///   the directory entry (propagated on Unix; best-effort on Windows where
///   opening a directory as a file is not always supported).
/// - No file locking is performed; concurrent writers may interleave whole
///   files (last rename wins).
///
/// All errors are wrapped with the destination path while preserving the
/// original [`io::ErrorKind`].
pub fn atomic_write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    let path = path.as_ref();
    let bytes = bytes.as_ref();
    inner(path, bytes).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("atomic_write {}: {error}", path.display()),
        )
    })
}

fn inner(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    fs::create_dir_all(parent)?;

    #[cfg(unix)]
    let prev_mode = fs::metadata(path)
        .ok()
        .map(|meta| meta.permissions().mode() & 0o7777);

    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.flush()?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|error| error.error)?;

    #[cfg(unix)]
    if let Some(mode) = prev_mode {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }

    sync_parent_dir(parent)
}

#[cfg(unix)]
fn sync_parent_dir(parent: &Path) -> io::Result<()> {
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}

#[cfg(not(unix))]
fn sync_parent_dir(parent: &Path) -> io::Result<()> {
    // Windows: opening a directory with `File::open` is not reliably
    // supported, so directory fsync is best-effort here. Failure to sync the
    // directory must not fail the write, since the file data itself was
    // already synced and renamed.
    if let Ok(dir) = fs::File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
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

    #[cfg(unix)]
    #[test]
    fn preserves_existing_mode() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("keys").join("secret");

        atomic_write(&path, b"first").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();

        atomic_write(&path, b"second").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "second");
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o7777;
        assert_eq!(mode, 0o644, "replace must preserve existing mode bits");
    }

    #[test]
    fn creates_nested_parent() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("a").join("b").join("c").join("state.toml");

        atomic_write(&path, b"nested").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "nested");
        assert!(path.parent().unwrap().is_dir());
    }

    #[test]
    fn empty_bytes_roundtrip() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("empty.bin");

        atomic_write(&path, b"").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"");
    }
}
