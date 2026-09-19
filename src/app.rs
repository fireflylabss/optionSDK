use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};

use crate::migrate::migrate_dir;
use crate::paths::{home_dir, option_root};

/// An Option family application.
///
/// Known apps are available as associated constants (`App::OPSH`, …).
/// Custom / experimental apps use [`App::new`], which also accepts
/// owned `String` ids (e.g. from config or CLI).
///
/// `App` is [`Clone`] but not [`Copy`] (identity strings are
/// `Cow<'static, str>` so dynamic ids work without leaking).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct App {
    id: Cow<'static, str>,
    mark: Cow<'static, str>,
    display_name: Cow<'static, str>,
    /// Path segment under `~/.option/` (usually same as `id`).
    dir_name: Cow<'static, str>,
    /// Relative legacy trees under `$HOME` to migrate into `dir()` once.
    /// Example: `[".optionos"]` or `["option", "music"]`.
    /// Each entry must be a single path segment (no `/` inside).
    legacy_home_parts: &'static [&'static str],
}

impl App {
    pub const OPSH: Self = Self {
        id: Cow::Borrowed("opsh"),
        mark: Cow::Borrowed("◆"),
        display_name: Cow::Borrowed("opsh"),
        dir_name: Cow::Borrowed("opsh"),
        legacy_home_parts: &[],
    };

    pub const TERMINAL: Self = Self {
        id: Cow::Borrowed("terminal"),
        mark: Cow::Borrowed("◇"),
        display_name: Cow::Borrowed("optionTerm"),
        dir_name: Cow::Borrowed("terminal"),
        legacy_home_parts: &[],
    };

    pub const MUSIC: Self = Self {
        id: Cow::Borrowed("music"),
        mark: Cow::Borrowed("♪"),
        display_name: Cow::Borrowed("optionMusic"),
        dir_name: Cow::Borrowed("music"),
        legacy_home_parts: &["option", "music"],
    };

    pub const FILES: Self = Self {
        id: Cow::Borrowed("files"),
        mark: Cow::Borrowed("◆"),
        display_name: Cow::Borrowed("optionFiles"),
        dir_name: Cow::Borrowed("files"),
        legacy_home_parts: &[],
    };

    pub const OS: Self = Self {
        id: Cow::Borrowed("os"),
        mark: Cow::Borrowed("◇"),
        display_name: Cow::Borrowed("optionOS"),
        dir_name: Cow::Borrowed("os"),
        legacy_home_parts: &[".optionos"],
    };

    pub const DE: Self = Self {
        id: Cow::Borrowed("de"),
        mark: Cow::Borrowed("◇"),
        display_name: Cow::Borrowed("optionDE"),
        dir_name: Cow::Borrowed("de"),
        legacy_home_parts: &[".optionde"],
    };

    pub const FAT: Self = Self {
        id: Cow::Borrowed("fat"),
        mark: Cow::Borrowed("◆"),
        display_name: Cow::Borrowed("fat"),
        dir_name: Cow::Borrowed("fat"),
        legacy_home_parts: &[],
    };

    pub const NOTES: Self = Self {
        id: Cow::Borrowed("notes"),
        mark: Cow::Borrowed("◇"),
        display_name: Cow::Borrowed("optionNotes"),
        dir_name: Cow::Borrowed("notes"),
        legacy_home_parts: &[".config", "optionnotes"],
    };

    pub const CAL: Self = Self {
        id: Cow::Borrowed("cal"),
        mark: Cow::Borrowed("◷"),
        display_name: Cow::Borrowed("optionCalendar"),
        dir_name: Cow::Borrowed("cal"),
        legacy_home_parts: &[],
    };

    pub const SEARCH: Self = Self {
        id: Cow::Borrowed("search"),
        mark: Cow::Borrowed("⌕"),
        display_name: Cow::Borrowed("optionSearch"),
        dir_name: Cow::Borrowed("search"),
        legacy_home_parts: &[],
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
        Self::CAL,
        Self::SEARCH,
    ];

    /// Build a custom app identity (no built-in legacy migrate).
    ///
    /// Accepts `&'static str` or owned `String` (e.g. from config/CLI).
    /// `dir_name` is set to `id`.
    pub fn new(
        id: impl Into<Cow<'static, str>>,
        mark: impl Into<Cow<'static, str>>,
        display_name: impl Into<Cow<'static, str>>,
    ) -> Self {
        let id = id.into();
        let dir_name = id.clone();
        Self {
            id,
            mark: mark.into(),
            display_name: display_name.into(),
            dir_name,
            legacy_home_parts: &[],
        }
    }

    /// Look up a known app by id (`"music"`, `"opsh"`, …).
    /// `"needle"` resolves to `SEARCH` as a legacy alias of the rename.
    pub fn known(id: &str) -> Option<Self> {
        if id == "needle" {
            return Some(Self::SEARCH);
        }
        Self::ALL.iter().find(|app| app.id() == id).cloned()
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn mark(&self) -> &str {
        &self.mark
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Reverse-DNS bundle id, e.g. `io.option.music`.
    pub fn bundle_id(&self) -> String {
        format!("io.option.{}", self.id)
    }

    /// `~/.option/<dir_name>`
    pub fn dir(&self) -> PathBuf {
        option_root().join(self.dir_name.as_ref() as &str)
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
        self.try_path("keys.toml").expect("static name")
    }

    /// `~/.option/<dir_name>/session.toml`
    pub fn session_toml(&self) -> PathBuf {
        self.try_path("session.toml").expect("static name")
    }

    /// Join a relative path under the app directory.
    ///
    /// The caller must pass a relative path with no `..` components.
    /// For untrusted input use [`Self::try_path`], which enforces this
    /// containment contract and returns an error instead of joining.
    pub fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.dir().join(relative)
    }

    /// Checked variant of [`Self::path`] for untrusted input.
    ///
    /// Rejects absolute paths and any `..` component so the result
    /// stays under the app directory. Internal callers (`keys_toml`,
    /// `session_toml`) route through here with static names.
    pub fn try_path(&self, relative: impl AsRef<Path>) -> std::io::Result<PathBuf> {
        let rel = relative.as_ref();
        if rel.is_absolute() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "absolute path not allowed",
            ));
        }
        if rel.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "parent-dir component not allowed",
            ));
        }
        Ok(self.dir().join(rel))
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
        assert_eq!(App::known("search"), Some(App::SEARCH));
        assert_eq!(App::known("needle"), Some(App::SEARCH));
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
        assert_eq!(App::SEARCH.bundle_id(), "io.option.search");
        assert_eq!(App::SEARCH.mark(), "⌕");
        assert_eq!(App::SEARCH.display_name(), "optionSearch");
    }

    #[test]
    fn custom_app() {
        let app = App::new("labs", "◇", "Option Labs");
        assert_eq!(app.id(), "labs");
        assert!(app.dir().ends_with(Path::new(".option").join("labs")));
    }

    #[test]
    fn dynamic_app_id() {
        let _guard = crate::test_env::lock();
        let root = tempfile::tempdir().unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("OPTION_HOME", root.path());
        }
        let id = String::from("dyn-") + "x";
        let app = App::new(id, String::from("◇"), String::from("Dyn X"));
        assert_eq!(app.id(), "dyn-x");
        assert!(app.dir().ends_with("dyn-x"));
        assert_eq!(app.dir(), root.path().join("dyn-x"));
        unsafe {
            std::env::remove_var("OPTION_HOME");
        }
    }

    #[test]
    fn try_path_ok() {
        let _guard = crate::test_env::lock();
        let root = tempfile::tempdir().unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("OPTION_HOME", root.path());
        }
        let dir = App::FILES.dir();
        assert_eq!(
            App::FILES.try_path("keys.toml").unwrap(),
            dir.join("keys.toml")
        );
        assert_eq!(App::FILES.try_path("a/b").unwrap(), dir.join("a/b"));
        assert_eq!(App::FILES.try_path("").unwrap(), dir.join(""));
        unsafe {
            std::env::remove_var("OPTION_HOME");
        }
    }

    #[test]
    fn try_path_rejects() {
        let _guard = crate::test_env::lock();
        let root = tempfile::tempdir().unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("OPTION_HOME", root.path());
        }
        assert!(App::FILES.try_path("/abs").is_err());
        assert!(App::FILES.try_path("../x").is_err());
        assert!(App::FILES.try_path("a/../../x").is_err());
        unsafe {
            std::env::remove_var("OPTION_HOME");
        }
    }

    #[test]
    fn notes_legacy_path_segments() {
        let _guard = crate::test_env::lock();
        let home = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let legacy = home.path().join(".config").join("optionnotes");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("keep.txt"), b"ok").unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("HOME", home.path());
            std::env::set_var("OPTION_HOME", root.path());
        }
        assert!(App::NOTES.migrate_legacy().unwrap());
        assert_eq!(
            std::fs::read_to_string(root.path().join("notes").join("keep.txt")).unwrap(),
            "ok"
        );
        assert!(!legacy.exists());
        unsafe {
            std::env::remove_var("OPTION_HOME");
            std::env::remove_var("HOME");
        }
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
