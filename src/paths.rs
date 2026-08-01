use std::path::{Path, PathBuf};

/// Resolve the user home directory.
///
/// Order: `$HOME`, then `$USERPROFILE` (Windows), else `.`.
pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// `~/.option` — shared root for all Option family apps.
///
/// When `OPTION_HOME` is set to a non-empty path, that value is used instead.
/// Useful for tests and disposable sandboxes without touching the real tree.
pub fn option_root() -> PathBuf {
    if let Some(root) = std::env::var_os("OPTION_HOME") {
        if !root.is_empty() {
            return PathBuf::from(root);
        }
    }
    home_dir().join(".option")
}

/// Expand a leading `~` or `~/…` using [`home_dir`].
///
/// Other paths are returned unchanged. Non-UTF-8 paths are returned as-is.
pub fn expand_tilde(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let Some(text) = path.to_str() else {
        return path.to_path_buf();
    };
    if text == "~" {
        return home_dir();
    }
    if let Some(rest) = text.strip_prefix("~/") {
        return home_dir().join(rest);
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn option_root_under_home() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::remove_var("OPTION_HOME");
            std::env::set_var("HOME", "/tmp/option-core-home");
        }
        assert_eq!(
            option_root(),
            Path::new("/tmp/option-core-home").join(".option")
        );
        unsafe {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn option_home_overrides_root() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("HOME", "/tmp/option-core-home");
            std::env::set_var("OPTION_HOME", "/tmp/option-sandbox");
        }
        assert_eq!(option_root(), Path::new("/tmp/option-sandbox"));
        unsafe {
            std::env::remove_var("OPTION_HOME");
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn empty_option_home_falls_through() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("HOME", "/tmp/option-core-home");
            std::env::set_var("OPTION_HOME", "");
        }
        assert_eq!(
            option_root(),
            Path::new("/tmp/option-core-home").join(".option")
        );
        unsafe {
            std::env::remove_var("OPTION_HOME");
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn home_dir_prefers_home_over_userprofile() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("HOME", "/tmp/from-home");
            std::env::set_var("USERPROFILE", "/tmp/from-userprofile");
        }
        assert_eq!(home_dir(), Path::new("/tmp/from-home"));
        unsafe {
            std::env::remove_var("HOME");
            std::env::remove_var("USERPROFILE");
        }
    }

    #[test]
    fn home_dir_falls_back_to_userprofile() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::remove_var("HOME");
            std::env::set_var("USERPROFILE", "/tmp/from-userprofile");
        }
        assert_eq!(home_dir(), Path::new("/tmp/from-userprofile"));
        unsafe {
            std::env::remove_var("USERPROFILE");
        }
    }

    #[test]
    fn expand_tilde_home_and_join() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("HOME", "/tmp/option-core-home");
        }
        assert_eq!(expand_tilde("~"), Path::new("/tmp/option-core-home"));
        assert_eq!(
            expand_tilde("~/Music"),
            Path::new("/tmp/option-core-home").join("Music")
        );
        assert_eq!(expand_tilde("/abs"), Path::new("/abs"));
        assert_eq!(expand_tilde("relative"), Path::new("relative"));
        unsafe {
            std::env::remove_var("HOME");
        }
    }
}
