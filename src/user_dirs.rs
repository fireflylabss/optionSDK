//! XDG base directories and localized user folders, without the `dirs` crate.
//!
//! Environment variables follow the XDG spec: an empty value counts as unset
//! and only absolute paths are honored. On Unix, the user folders
//! (`documents`, `music`, `downloads`, `pictures`) also consult
//! `$XDG_CONFIG_HOME/user-dirs.dirs` so localized names like `Documentos`
//! resolve correctly; the file is read on every call (no global cache), so
//! tests can point `HOME` at a tempdir.

use std::path::PathBuf;

/// `$XDG_DATA_HOME` or `~/.local/share`.
pub fn data_home() -> PathBuf {
    xdg_env("XDG_DATA_HOME").unwrap_or_else(|| crate::home_dir().join(".local/share"))
}

/// `$XDG_CONFIG_HOME` or `~/.config`.
pub fn config_home() -> PathBuf {
    xdg_env("XDG_CONFIG_HOME").unwrap_or_else(|| crate::home_dir().join(".config"))
}

/// `$XDG_CACHE_HOME` or `~/.cache`.
pub fn cache_home() -> PathBuf {
    xdg_env("XDG_CACHE_HOME").unwrap_or_else(|| crate::home_dir().join(".cache"))
}

/// `XDG_DOCUMENTS_DIR` or `~/Documents`.
pub fn documents() -> PathBuf {
    user_dir("XDG_DOCUMENTS_DIR", "Documents")
}

/// `XDG_MUSIC_DIR` or `~/Music`.
pub fn music() -> PathBuf {
    user_dir("XDG_MUSIC_DIR", "Music")
}

/// `XDG_DOWNLOAD_DIR` or `~/Downloads`.
pub fn downloads() -> PathBuf {
    user_dir("XDG_DOWNLOAD_DIR", "Downloads")
}

/// `XDG_PICTURES_DIR` or `~/Pictures`.
pub fn pictures() -> PathBuf {
    user_dir("XDG_PICTURES_DIR", "Pictures")
}

/// An absolute, non-empty env var value, per the XDG base-dir spec.
fn xdg_env(name: &str) -> Option<PathBuf> {
    let value = std::env::var_os(name)?;
    if value.is_empty() {
        return None;
    }
    let path = PathBuf::from(value);
    path.is_absolute().then_some(path)
}

fn user_dir(env_name: &str, fallback: &str) -> PathBuf {
    if let Some(path) = xdg_env(env_name) {
        return path;
    }
    #[cfg(unix)]
    if let Some(path) = user_dirs_file(env_name) {
        return path;
    }
    crate::home_dir().join(fallback)
}

/// Look up `env_name` in `config_home()/user-dirs.dirs`.
#[cfg(unix)]
fn user_dirs_file(env_name: &str) -> Option<PathBuf> {
    let text = std::fs::read_to_string(config_home().join("user-dirs.dirs")).ok()?;
    parse_user_dirs(&text, env_name)
}

/// Parse lines like `XDG_DOCUMENTS_DIR="$HOME/Documentos"`.
///
/// A leading `$HOME` resolves via [`crate::home_dir`]. Lines without `=`,
/// with unbalanced quotes, or resolving to a relative path are skipped.
#[cfg(unix)]
fn parse_user_dirs(text: &str, env_name: &str) -> Option<PathBuf> {
    let prefix = format!("{env_name}=");
    for line in text.lines() {
        let Some(value) = line.trim().strip_prefix(&prefix) else {
            continue;
        };
        let value = value.trim();
        let value = if let Some(quoted) = value.strip_prefix('"') {
            let Some(inner) = quoted.strip_suffix('"') else {
                continue;
            };
            inner
        } else {
            value
        };
        let expanded = if value == "$HOME" {
            crate::home_dir()
        } else if let Some(rest) = value.strip_prefix("$HOME/") {
            crate::home_dir().join(rest.trim_start_matches('/'))
        } else {
            PathBuf::from(value)
        };
        if expanded.is_absolute() {
            return Some(expanded);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env;
    use std::ffi::OsString;
    use std::path::Path;

    /// Points `HOME` + `XDG_CONFIG_HOME` at a tempdir and clears the XDG vars
    /// tests rely on, so the `user-dirs.dirs` lookup is deterministic. Every
    /// saved value is restored on drop; the TempDir is owned so it outlives
    /// the test.
    struct Sandbox {
        dir: tempfile::TempDir,
        saved: Vec<(&'static str, Option<OsString>)>,
    }

    impl Sandbox {
        const VARS: &[&'static str] = &[
            "HOME",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "XDG_DOCUMENTS_DIR",
            "XDG_MUSIC_DIR",
            "XDG_DOWNLOAD_DIR",
            "XDG_PICTURES_DIR",
        ];

        fn new() -> Self {
            let saved = Self::VARS
                .iter()
                .map(|name| (*name, std::env::var_os(name)))
                .collect();
            let dir = tempfile::tempdir().unwrap();
            // SAFETY: tests serialize env mutation via ENV_LOCK.
            unsafe {
                std::env::set_var("HOME", dir.path());
                std::env::set_var("XDG_CONFIG_HOME", dir.path().join(".config"));
                for name in &Self::VARS[2..] {
                    std::env::remove_var(name);
                }
            }
            Self { dir, saved }
        }

        fn path(&self) -> &Path {
            self.dir.path()
        }
    }

    impl Drop for Sandbox {
        fn drop(&mut self) {
            // SAFETY: tests serialize env mutation via ENV_LOCK; the guard is
            // dropped before `_guard`, so the lock is still held.
            unsafe {
                for (name, value) in &self.saved {
                    match value {
                        Some(value) => std::env::set_var(name, value),
                        None => std::env::remove_var(name),
                    }
                }
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn user_dirs_file_resolves_home() {
        let _guard = test_env::lock();
        let home = Sandbox::new();
        let config = home.path().join(".config");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(
            config.join("user-dirs.dirs"),
            "XDG_DESKTOP_DIR=\"$HOME/Desktop\"\n\
             XDG_DOCUMENTS_DIR=\"$HOME/Documentos\"\n\
             XDG_MUSIC_DIR=\"$HOME/Musicas\"\n",
        )
        .unwrap();
        assert_eq!(documents(), home.path().join("Documentos"));
        assert_eq!(music(), home.path().join("Musicas"));
        // No downloads entry → English fallback under the temp HOME.
        assert_eq!(downloads(), home.path().join("Downloads"));
    }

    #[cfg(unix)]
    #[test]
    fn user_dirs_file_skips_malformed() {
        let _guard = test_env::lock();
        let home = Sandbox::new();
        let config = home.path().join(".config");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(
            config.join("user-dirs.dirs"),
            "garbage line\n\
             XDG_DOCUMENTS_DIR=\"unbalanced\n\
             XDG_MUSIC_DIR=relative/path\n",
        )
        .unwrap();
        assert_eq!(documents(), home.path().join("Documents"));
        assert_eq!(music(), home.path().join("Music"));
    }

    #[test]
    fn missing_file_falls_back() {
        let _guard = test_env::lock();
        let home = Sandbox::new();
        assert_eq!(documents(), home.path().join("Documents"));
        assert_eq!(pictures(), home.path().join("Pictures"));
    }

    #[test]
    fn env_var_wins_when_absolute() {
        let _guard = test_env::lock();
        let _home = Sandbox::new();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("XDG_DOCUMENTS_DIR", "/srv/docs");
        }
        assert_eq!(documents(), PathBuf::from("/srv/docs"));
    }

    #[test]
    fn relative_and_empty_env_ignored() {
        let _guard = test_env::lock();
        let home = Sandbox::new();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("XDG_DATA_HOME", "relative/share");
        }
        assert_eq!(data_home(), home.path().join(".local/share"));
        unsafe {
            std::env::set_var("XDG_DATA_HOME", "");
        }
        assert_eq!(data_home(), home.path().join(".local/share"));
    }

    #[test]
    fn base_dirs_use_env() {
        let _guard = test_env::lock();
        let home = Sandbox::new();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("XDG_DATA_HOME", "/xdg/data");
            std::env::set_var("XDG_CACHE_HOME", "/xdg/cache");
        }
        assert_eq!(data_home(), PathBuf::from("/xdg/data"));
        assert_eq!(config_home(), home.path().join(".config"));
        assert_eq!(cache_home(), PathBuf::from("/xdg/cache"));
    }
}
