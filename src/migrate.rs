use std::fs;
use std::io;
use std::path::Path;

/// Move `legacy` → `new` once when `new` is missing and `legacy` exists.
///
/// Creates the parent of `new` when needed. Returns `Ok(true)` if a rename ran.
pub fn migrate_dir(legacy: &Path, new: &Path) -> io::Result<bool> {
    if new.exists() || !legacy.exists() {
        return Ok(false);
    }
    if let Some(parent) = new.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(legacy, new)?;
    Ok(true)
}

/// Move a single file `legacy` → `new` once when `new` is missing and `legacy` exists.
///
/// Creates the parent of `new` when needed. Returns `Ok(true)` if a rename ran.
pub fn migrate_file(legacy: &Path, new: &Path) -> io::Result<bool> {
    if new.exists() || !legacy.exists() || !legacy.is_file() {
        return Ok(false);
    }
    if let Some(parent) = new.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(legacy, new)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn migrate_dir_moves_once() {
        let root = tempdir().unwrap();
        let legacy = root.path().join("old");
        let new = root.path().join("nest").join("new");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("keep.txt"), b"ok").unwrap();

        assert!(migrate_dir(&legacy, &new).unwrap());
        assert!(new.join("keep.txt").is_file());
        assert!(!legacy.exists());
        assert!(!migrate_dir(&legacy, &new).unwrap());
    }

    #[test]
    fn migrate_file_moves_once() {
        let root = tempdir().unwrap();
        let legacy = root.path().join("old.txt");
        let new = root.path().join("a").join("b").join("new.txt");
        fs::write(&legacy, b"hi").unwrap();

        assert!(migrate_file(&legacy, &new).unwrap());
        assert_eq!(fs::read_to_string(&new).unwrap(), "hi");
        assert!(!legacy.exists());
        assert!(!migrate_file(&legacy, &new).unwrap());
    }

    #[test]
    fn skips_when_new_exists() {
        let root = tempdir().unwrap();
        let legacy = root.path().join("legacy");
        let new = root.path().join("new");
        fs::create_dir_all(&legacy).unwrap();
        fs::create_dir_all(&new).unwrap();
        fs::write(legacy.join("a"), b"1").unwrap();
        fs::write(new.join("b"), b"2").unwrap();

        assert!(!migrate_dir(&legacy, &new).unwrap());
        assert!(legacy.join("a").is_file());
        assert!(new.join("b").is_file());
    }
}
