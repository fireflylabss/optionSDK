# Changelog

All notable changes to this project will be documented in this file.

Versioning, surfaces, and channels: see [VERSIONING.md](VERSIONING.md).

## [0.1.1-stable] - 2026-08-01

### Added

- `color_on_stderr()` — `NO_COLOR` plus stderr is a TTY (mirror of `color_on_stdout()`).

### Changed

- Tests that mutate process environment now share one crate-wide `ENV_LOCK`, so parallel test threads cannot race on `$HOME` / `OPTION_HOME` / `NO_COLOR`.

## [0.1.0-stable] - 2026-08-01

### Added

- First crates.io release of **optionSDK** (Rust crate name `option_sdk`).
- App identity registry: `opsh`, `terminal`, `music`, `files`, `os`, `de`, `fat`, `notes` — mark, display name, `io.option.*` bundle id.
- Paths: `home_dir()`, `option_root()`, `expand_tilde()`, `App::dir()`, `config_toml()`, `cache_dir()`, `keys_toml()`, `session_toml()`, `path()`.
- `OPTION_HOME` override for the shared root (tests / sandboxes).
- `App::ensure()` / `App::ensure_cache()` with known legacy-tree migrate.
- `migrate_dir()` / `migrate_file()` helpers.
- Color: `color_enabled()` (`NO_COLOR`), `color_on_stdout()` (`NO_COLOR` + TTY).
- CI workflow (fmt, test, release build).
