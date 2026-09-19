//! Typed TOML config persistence for Option family apps.
//!
//! `~/.option/<dir>/config.toml` is the canonical per-app config file.
//! [`load_toml`] / [`save_toml`] work on explicit paths; [`App::load_config`]
//! / [`App::save_config`] wrap them for the app directory.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::App;

/// Errors from TOML config load/save.
#[derive(Debug)]
pub enum ConfigError {
    /// Filesystem error (already carries the path via [`crate::atomic_write`]).
    Io(io::Error),
    /// The file exists but is not valid TOML for `T`.
    Parse {
        /// Path of the offending file.
        path: PathBuf,
        /// TOML deserialization error.
        source: toml::de::Error,
    },
    /// `T` failed to serialize to TOML.
    Serialize(toml::ser::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Parse { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Serialize(source) => write!(f, "{source}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Parse { source, .. } => Some(source),
            Self::Serialize(source) => Some(source),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Load a TOML file as `T`.
///
/// Returns `Ok(None)` when the file does not exist; a file that exists but
/// fails to parse is a [`ConfigError::Parse`].
pub fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, ConfigError> {
    if !path.exists() {
        return Ok(None);
    }
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let value = toml::from_str(&text).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(Some(value))
}

/// Load the first existing file from `paths`, returning the winning path too.
///
/// Useful for legacy config locations: order `paths` from newest to oldest.
pub fn load_toml_chain<T: DeserializeOwned>(
    paths: &[PathBuf],
) -> Result<Option<(T, PathBuf)>, ConfigError> {
    for path in paths {
        if path.exists() {
            return load_toml(path).map(|value| value.map(|v| (v, path.clone())));
        }
    }
    Ok(None)
}

/// Serialize `value` as pretty TOML and persist it via [`crate::atomic_write`].
pub fn save_toml<T: Serialize>(path: &Path, value: &T) -> Result<(), ConfigError> {
    let text = toml::to_string_pretty(value).map_err(ConfigError::Serialize)?;
    crate::atomic_write(path, text.as_bytes())?;
    Ok(())
}

impl App {
    /// Load `config.toml`, writing `T::default()` when the file is missing.
    ///
    /// Runs [`App::ensure`] first, so the app directory and legacy migration
    /// are handled before the file is touched.
    pub fn load_config<T: DeserializeOwned + Serialize + Default>(&self) -> Result<T, ConfigError> {
        self.ensure()?;
        let path = self.config_toml();
        match load_toml(&path)? {
            Some(value) => Ok(value),
            None => {
                let value = T::default();
                save_toml(&path, &value)?;
                Ok(value)
            }
        }
    }

    /// Persist `value` to `config.toml` (via [`crate::atomic_write`]).
    pub fn save_config<T: Serialize>(&self, value: &T) -> Result<(), ConfigError> {
        self.ensure()?;
        save_toml(&self.config_toml(), value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
    struct Settings {
        name: String,
        count: u32,
        enabled: bool,
    }

    fn sample() -> Settings {
        Settings {
            name: "option".to_string(),
            count: 3,
            enabled: true,
        }
    }

    #[test]
    fn roundtrip() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        save_toml(&path, &sample()).unwrap();
        assert_eq!(load_toml::<Settings>(&path).unwrap(), Some(sample()));
    }

    #[test]
    fn missing_file_is_none() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("nope.toml");
        assert_eq!(load_toml::<Settings>(&path).unwrap(), None);
    }

    #[test]
    fn invalid_toml_reports_path() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("broken.toml");
        std::fs::write(&path, "this is = [not toml").unwrap();
        match load_toml::<Settings>(&path) {
            Err(ConfigError::Parse { path: p, .. }) => {
                assert_eq!(p, path);
            }
            other => panic!("expected Parse error, got {other:?}"),
        }
        let err = load_toml::<Settings>(&path).unwrap_err();
        assert!(err.to_string().contains(path.to_str().unwrap()));
    }

    #[test]
    fn chain_picks_first_existing() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("a.toml");
        let second = root.path().join("b.toml");
        let third = root.path().join("c.toml");
        save_toml(&second, &sample()).unwrap();
        save_toml(&third, &Settings::default()).unwrap();
        let (value, winner) =
            load_toml_chain::<Settings>(&[first.clone(), second.clone(), third.clone()])
                .unwrap()
                .unwrap();
        assert_eq!(value, sample());
        assert_eq!(winner, second);
        assert_eq!(load_toml_chain::<Settings>(&[first]).unwrap(), None);
    }

    #[test]
    fn load_config_writes_defaults() {
        let _guard = test_env::lock();
        let root = tempfile::tempdir().unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("OPTION_HOME", root.path());
        }
        let app = App::OPSH;
        let path = app.config_toml();
        assert!(!path.exists());
        let settings = app.load_config::<Settings>().unwrap();
        assert_eq!(settings, Settings::default());
        assert!(path.is_file());
        // Second call reads the file back instead of rewriting.
        assert_eq!(app.load_config::<Settings>().unwrap(), Settings::default());
        app.save_config(&sample()).unwrap();
        assert_eq!(app.load_config::<Settings>().unwrap(), sample());
        unsafe {
            std::env::remove_var("OPTION_HOME");
        }
    }
}
