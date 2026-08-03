use std::path::{Path, PathBuf};

use crate::migrate::migrate_dir;
use crate::paths::{home_dir, option_root};

/// An Option family application.
///
/// Known apps are available as associated constants (`App::OPSH`, …).
/// Custom / experimental apps use [`App::new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct App {
    id: &'static str,
    mark: &'static str,
    display_name: &'static str,
    /// Path segment under `~/.option/` (usually same as `id`).
    dir_name: &'static str,
    /// Relative legacy trees under `$HOME` to migrate into `dir()` once.
    /// Example: `[".optionos"]` or `["option", "music"]`.
    legacy_home_parts: &'static [&'static str],
}

impl App {
    pub const OPSH: Self = Self {
        id: "opsh",
        mark: "◆",
        display_name: "opsh",
        dir_name: "opsh",
        legacy_home_parts: &[],
    };

    pub const TERMINAL: Self = Self {
        id: "terminal",
        mark: "◇",
        display_name: "optionTerm",
        dir_name: "terminal",
        legacy_home_parts: &[],
    };

    pub const MUSIC: Self = Self {
        id: "music",
        mark: "♪",
        display_name: "optionMusic",
        dir_name: "music",
        legacy_home_parts: &["option", "music"],
    };

    pub const FILES: Self = Self {
        id: "files",
        mark: "◆",
        display_name: "optionFiles",
        dir_name: "files",
        legacy_home_parts: &[],
    };

    pub const OS: Self = Self {
        id: "os",
        mark: "◇",
        display_name: "optionOS",
        dir_name: "os",
        legacy_home_parts: &[".optionos"],
    };

    pub const DE: Self = Self {
        id: "de",
        mark: "◇",
        display_name: "optionDE",
        dir_name: "de",
        legacy_home_parts: &[".optionde"],
    };

    pub const FAT: Self = Self {
        id: "fat",
        mark: "◆",
        display_name: "fat",
        dir_name: "fat",
        legacy_home_parts: &[],
    };

    pub const NOTES: Self = Self {
        id: "notes",
        mark: "◇",
        display_name: "optionNotes",
        dir_name: "notes",
        legacy_home_parts: &[".config/optionnotes"],
    };

    /// All known family apps.
    pub const ALL: &'static [Self] = &[
        Self::OPSH,
        Self::TERMINAL,
        Self::MUSIC,
        Self::FILES,
        Self::OS,
        Self::DE,
        Self::FAT,
        Self::NOTES,
    ];

    /// Build a custom app identity (no built-in legacy migrate).
    pub const fn new(id: &'static str, mark: &'static str, display_name: &'static str) -> Self {
        Self {
            id,
            mark,
            display_name,
            dir_name: id,
            legacy_home_parts: &[],
        }
    }

    /// Look up a known app by id (`"music"`, `"opsh"`, …).
    pub fn known(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|app| app.id == id)
    }

    pub const fn id(&self) -> &'static str {
        self.id
    }

    pub const fn mark(&self) -> &'static str {
        self.mark
    }

    pub const fn display_name(&self) -> &'static str {
        self.display_name
    }

    /// Reverse-DNS bundle id, e.g. `io.option.music`.
    pub fn bundle_id(&self) -> String {
        format!("io.option.{}", self.id)
    }

    /// `~/.option/<dir_name>`
    pub fn dir(&self) -> PathBuf {
        option_root().join(self.dir_name)
    }

    /// `~/.option/<dir_name>/config.toml`
    pub fn config_toml(&self) -> PathBuf {
        self.dir().join("config.toml")
    }

    /// `~/.option/<dir_name>/cache`
    pub fn cache_dir(&self) -> PathBuf {
        self.dir().join("cache")
    }

    /// `~/.option/<dir_name>/keys.toml`
    pub fn keys_toml(&self) -> PathBuf {
        self.path("keys.toml")
    }

    /// `~/.option/<dir_name>/session.toml`
    pub fn session_toml(&self) -> PathBuf {
        self.path("session.toml")
    }

    /// Join a relative path under the app directory.
    pub fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.dir().join(relative)
    }

    /// Create the app directory (and `~/.option`), migrating any known legacy tree.
    ///
    /// Returns the canonical app directory path.
    pub fn ensure(&self) -> std::io::Result<PathBuf> {
        let dir = self.dir();
        self.migrate_legacy()?;
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// Ensure the app directory exists, then create `cache/` under it.
    ///
    /// Returns the cache directory path.
    pub fn ensure_cache(&self) -> std::io::Result<PathBuf> {
        self.ensure()?;
        let cache = self.cache_dir();
        std::fs::create_dir_all(&cache)?;
        Ok(cache)
    }

    /// Migrate a known legacy home-relative tree into [`Self::dir`] when missing.
    pub fn migrate_legacy(&self) -> std::io::Result<bool> {
        if self.legacy_home_parts.is_empty() {
            return Ok(false);
        }
        let home = home_dir();
        let legacy = self
            .legacy_home_parts
            .iter()
            .fold(home, |acc, part| acc.join(part));
        migrate_dir(&legacy, &self.dir())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_lookup() {
        assert_eq!(App::known("opsh"), Some(App::OPSH));
        assert_eq!(App::known("notes"), Some(App::NOTES));
        assert_eq!(App::known("nope"), None);
    }

    #[test]
    fn bundle_ids() {
        assert_eq!(App::MUSIC.bundle_id(), "io.option.music");
        assert_eq!(App::TERMINAL.bundle_id(), "io.option.terminal");
        assert_eq!(App::NOTES.bundle_id(), "io.option.notes");
        assert_eq!(App::OPSH.mark(), "◆");
        assert_eq!(App::MUSIC.mark(), "♪");
        assert_eq!(App::MUSIC.display_name(), "optionMusic");
        assert_eq!(App::NOTES.mark(), "◇");
        assert_eq!(App::NOTES.display_name(), "optionNotes");
    }

    #[test]
    fn custom_app() {
        let app = App::new("labs", "◇", "Option Labs");
        assert_eq!(app.id(), "labs");
        assert!(app.dir().ends_with(Path::new(".option").join("labs")));
    }

    #[test]
    fn well_known_paths() {
        assert_eq!(
            App::TERMINAL
                .keys_toml()
                .file_name()
                .and_then(|s| s.to_str()),
            Some("keys.toml")
        );
        assert_eq!(
            App::TERMINAL
                .session_toml()
                .file_name()
                .and_then(|s| s.to_str()),
            Some("session.toml")
        );
        assert_eq!(
            App::MUSIC.cache_dir().file_name().and_then(|s| s.to_str()),
            Some("cache")
        );
    }

    #[test]
    fn ensure_cache_creates_tree() {
        let _guard = crate::test_env::lock();
        let root = tempfile::tempdir().unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("OPTION_HOME", root.path());
        }
        let cache = App::FILES.ensure_cache().unwrap();
        assert!(cache.is_dir());
        assert_eq!(cache, root.path().join("files").join("cache"));
        unsafe {
            std::env::remove_var("OPTION_HOME");
        }
    }
}
