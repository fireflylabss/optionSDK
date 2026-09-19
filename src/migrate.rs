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
    ensure_parent(new).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("migrate {} -> {}: {e}", legacy.display(), new.display()),
        )
    })?;
    rename_with_context(legacy, new)?;
    Ok(true)
}

/// Move a single file `legacy` → `new` once when `new` is missing and `legacy` exists.
///
/// Creates the parent of `new` when needed. Returns `Ok(true)` if a rename ran.
pub fn migrate_file(legacy: &Path, new: &Path) -> io::Result<bool> {
    if new.exists() || !legacy.exists() || !legacy.is_file() {
        return Ok(false);
    }
    ensure_parent(new).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("migrate {} -> {}: {e}", legacy.display(), new.display()),
        )
    })?;
    rename_with_context(legacy, new)?;
    Ok(true)
}

fn ensure_parent(new: &Path) -> io::Result<()> {
    if let Some(p) = new.parent() {
        fs::create_dir_all(p)
            .map_err(|e| io::Error::new(e.kind(), format!("create parent {}: {e}", p.display())))?;
    }
    Ok(())
}

fn is_cross_device(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::CrossesDevices || e.raw_os_error() == Some(18)
}

fn rename_with_context(legacy: &Path, new: &Path) -> io::Result<()> {
    match fs::rename(legacy, new) {
        Ok(()) => Ok(()),
        Err(e) if is_cross_device(&e) => {
            let ctx = |inner: io::Error| {
                io::Error::new(
                    inner.kind(),
                    format!("migrate {} -> {}: {inner}", legacy.display(), new.display()),
                )
            };
            let meta = fs::symlink_metadata(legacy).map_err(ctx)?;
            if meta.is_dir() && !meta.file_type().is_symlink() {
                copy_dir_recursive(legacy, new).map_err(ctx)?;
                fs::remove_dir_all(legacy).map_err(ctx)?;
            } else {
                fs::copy(legacy, new).map_err(ctx)?;
                #[cfg(unix)]
                {
                    if let Ok(perm) = fs::metadata(legacy).map(|m| m.permissions()) {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = fs::set_permissions(new, fs::Permissions::from_mode(perm.mode()));
                    }
                }
                fs::remove_file(legacy).map_err(ctx)?;
            }
            Ok(())
        }
        Err(e) => Err(io::Error::new(
            e.kind(),
            format!("migrate {} -> {}: {e}", legacy.display(), new.display()),
        )),
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let ft = entry.file_type()?;
        if ft.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else if ft.is_symlink() {
            #[cfg(unix)]
            {
                let target = fs::read_link(&src_path)?;
                std::os::unix::fs::symlink(target, &dst_path)?;
            }
            #[cfg(not(unix))]
            {
                fs::copy(&src_path, &dst_path)?;
            }
        } else {
            fs::copy(&src_path, &dst_path)?;
            #[cfg(unix)]
            {
                if let Ok(perm) = fs::metadata(&src_path).map(|m| m.permissions()) {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&dst_path, fs::Permissions::from_mode(perm.mode()));
                }
            }
        }
    }
    #[cfg(unix)]
    {
        if let Ok(perm) = fs::metadata(src).map(|m| m.permissions()) {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(dst, fs::Permissions::from_mode(perm.mode()));
        }
    }
    Ok(())
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

    #[test]
    fn error_context_includes_both_paths() {
        let root = tempdir().unwrap();
        let legacy = root.path().join("legacy");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("a"), b"1").unwrap();
        // Block parent creation: `blocker` is a file, so `blocker/new` cannot be created.
        let blocker = root.path().join("blocker");
        fs::write(&blocker, b"x").unwrap();
        let new = blocker.join("new");

        let err = migrate_dir(&legacy, &new).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains(&legacy.display().to_string()),
            "missing legacy: {msg}"
        );
        assert!(
            msg.contains(&new.display().to_string()),
            "missing new: {msg}"
        );
    }

    #[test]
    fn copy_dir_recursive_preserves_tree() {
        let root = tempdir().unwrap();
        let src = root.path().join("src");
        let dst = root.path().join("dst");
        fs::create_dir_all(src.join("sub").join("deep")).unwrap();
        fs::write(src.join("top.txt"), b"top").unwrap();
        fs::write(src.join("sub").join("mid.txt"), b"mid").unwrap();
        fs::write(src.join("sub").join("deep").join("leaf.txt"), b"leaf").unwrap();

        copy_dir_recursive(&src, &dst).unwrap();

        assert_eq!(fs::read_to_string(dst.join("top.txt")).unwrap(), "top");
        assert_eq!(
            fs::read_to_string(dst.join("sub").join("mid.txt")).unwrap(),
            "mid"
        );
        assert_eq!(
            fs::read_to_string(dst.join("sub").join("deep").join("leaf.txt")).unwrap(),
            "leaf"
        );
        // Source untouched: copy must not delete.
        assert!(src.join("top.txt").is_file());
    }
}
