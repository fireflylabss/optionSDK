use std::path::{Path, PathBuf};

/// Resolve the user home directory.
///
/// Order: explicit `$HOME` first (tests/sandboxes depend on it), then
/// explicit `$USERPROFILE` (Windows), then [`home::home_dir`] (covers
/// `HOMEDRIVE`/`HOMEPATH` and known folders on Windows).
///
/// Panics when the home directory cannot be determined. Set `HOME` or
/// `OPTION_HOME` in sandboxes/systemd units without a home.
pub fn home_dir() -> PathBuf {
    home_dir_opt().expect("HOME and USERPROFILE are both unset; set HOME or OPTION_HOME")
}

/// Same as [`home_dir`], but returns `None` instead of panicking when the
/// home directory cannot be determined.
pub fn home_dir_opt() -> Option<PathBuf> {
    if let Some(h) = std::env::var_os("HOME").map(PathBuf::from) {
        return Some(h);
    }
    if let Some(h) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
        return Some(h);
    }
    home::home_dir()
}

/// `~/.option` — shared root for all Option family apps.
///
/// Canonical root is `~/.option` by design (not XDG); XDG migration is
/// explicitly deferred.
///
/// When `OPTION_HOME` is set to a non-empty path, that value is used instead.
/// Useful for tests and disposable sandboxes without touching the real tree.
///
/// Panics when neither `OPTION_HOME` nor a home directory is available;
/// set `HOME` or `OPTION_HOME`.
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
        let rest = rest.trim_start_matches(['/', '\\']);
        return home_dir().join(rest);
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env;
    use std::path::Path;

    #[test]
    fn option_root_under_home() {
        let _guard = test_env::lock();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::remove_var("OPTION_HOME");
            std::env::set_var("HOME", "/tmp/option-sdk-home");
        }
        assert_eq!(
            option_root(),
            Path::new("/tmp/option-sdk-home").join(".option")
        );
        unsafe {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn option_home_overrides_root() {
        let _guard = test_env::lock();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("HOME", "/tmp/option-sdk-home");
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
        let _guard = test_env::lock();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("HOME", "/tmp/option-sdk-home");
            std::env::set_var("OPTION_HOME", "");
        }
        assert_eq!(
            option_root(),
            Path::new("/tmp/option-sdk-home").join(".option")
        );
        unsafe {
            std::env::remove_var("OPTION_HOME");
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn home_dir_prefers_home_over_userprofile() {
        let _guard = test_env::lock();
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
        let _guard = test_env::lock();
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
        let _guard = test_env::lock();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("HOME", "/tmp/option-sdk-home");
        }
        assert_eq!(expand_tilde("~"), Path::new("/tmp/option-sdk-home"));
        assert_eq!(
            expand_tilde("~/Music"),
            Path::new("/tmp/option-sdk-home").join("Music")
        );
        assert_eq!(expand_tilde("/abs"), Path::new("/abs"));
        assert_eq!(expand_tilde("relative"), Path::new("relative"));
        unsafe {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn expand_tilde_double_slash_stays_under_home() {
        let _guard = test_env::lock();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("HOME", "/tmp/option-sdk-home");
        }
        assert_eq!(
            expand_tilde("~//foo"),
            Path::new("/tmp/option-sdk-home").join("foo")
        );
        assert_eq!(expand_tilde("~/"), Path::new("/tmp/option-sdk-home"));
        unsafe {
            std::env::remove_var("HOME");
        }
    }

    #[test]
    fn home_dir_never_falls_back_to_cwd() {
        let _guard = test_env::lock();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::remove_var("HOME");
            std::env::remove_var("USERPROFILE");
        }
        // With the `home` crate (plan 011), the OS may still resolve a home
        // (e.g. via passwd); the invariant from plan 003 is that we never
        // silently fall back to CWD ("." or a relative path).
        if let Some(h) = home_dir_opt() {
            assert_ne!(h, PathBuf::from("."));
            assert!(h.is_absolute(), "home must be absolute, got {h:?}");
        }
    }

    #[test]
    fn option_root_option_home_rescues_when_home_unset() {
        let _guard = test_env::lock();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::remove_var("HOME");
            std::env::remove_var("USERPROFILE");
            std::env::set_var("OPTION_HOME", "/tmp/option-sandbox");
        }
        assert_eq!(option_root(), Path::new("/tmp/option-sandbox"));
        unsafe {
            std::env::remove_var("OPTION_HOME");
        }
    }

    #[test]
    fn home_dir_never_returns_cwd_dot() {
        let _guard = test_env::lock();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("HOME", "/tmp/option-sdk-home");
            std::env::remove_var("USERPROFILE");
        }
        // Loud-failure path lives in `home_dir()`'s expect() message; here we
        // pin the no-silent-CWD invariant on the compat wrapper itself.
        let h = home_dir();
        assert_ne!(h, PathBuf::from("."));
        assert!(h.is_absolute(), "home must be absolute, got {h:?}");
        unsafe {
            std::env::remove_var("HOME");
        }
    }
}
