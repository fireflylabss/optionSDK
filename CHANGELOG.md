# Changelog

We follow [Semantic Versioning](https://semver.org/) and [Keep a Changelog](https://keepachangelog.com/). optionSDK is the shared Rust crate on [crates.io](https://crates.io/crates/optionSDK) (`option_sdk` in Rust).

<details>
<summary>To see more about versioning, expand this.</summary>

Every version string starts with `v` (required), e.g. `v0.1.1`.

This project publishes **plain SemVer** to crates.io — no app surfaces, no `m` mixed tags, and no alpha/beta/stable channel suffixes in the tag. `Cargo.toml`, git tags, and crates.io use the same numeric version (`0.1.1` / `v0.1.1`).

Each release heading is the version and date (`## v0.1.3 · 03/08/2026`); under it, a short summary ends with a plain sentence like: “This version was made for the shared SDK on 03/08/2026 (v0.1.3).”

</details>

## v0.1.6 · 20/09/2026

Docs and test polish on top of the registry cleanup. This version was made for the shared SDK on 20/09/2026 (v0.1.6).

- Fix the README features heading to track the crate version.
- Cover `App::MUSIC`'s legacy-tree migration with a test again — the `notes` case left together with the app.

## v0.1.5 · 20/09/2026

Removes the deprecated `fat` and `notes` apps from the family registry, and gives opsh its own mark. This version was made for the shared SDK on 20/09/2026 (v0.1.5).

- Remove `App::FAT` and `App::NOTES` — both apps are deprecated and no longer ship; `App::ALL` now lists 8 apps.
- `App::OPSH` mark changes from `◆` to `❯`, echoing the shell's `›` prompt glyph and keeping every shipped app's mark unique.

## v0.1.4 · 19/09/2026

Adds optionCalendar and optionSearch to the family registry, plus shared config, crash, doctor, terminal style and user-dir helpers. This version was made for the shared SDK on 19/09/2026 (v0.1.4).

- Add `App::CAL` (`◷ optionCalendar`, `~/.option/cal/`) to the known family apps; required by optionCalendar v0.1.0.
- Add `App::SEARCH` (`search` / `⌕` / `optionSearch`, dir `search/`) to `App::ALL`; `App::known("needle")` resolves to `SEARCH` as a legacy alias of the rename.
- Config: `load_toml()`, `load_toml_chain()`, `save_toml()` plus `App::load_config()` / `App::save_config()` — typed TOML under `config.toml`, persisted via `atomic_write()`.
- Crash: `install_crash_hook()` and `crash_log_path()` — shared panic hook appending timestamped entries to `~/.option/<app>/crash.log`.
- Doctor: `doctor::Report` / `doctor::Check` / `doctor::Status` with `checks::{state_dir, file, binary}` — text and single-line JSON rendering for `doctor [--json]` commands.
- Term: `Style` — ANSI `bold`/`dim`/`ok`/`warn`/`err` and `mark_line()`; `Style::stdout()` / `Style::stderr()` pick up terminal detection.
- User dirs: `user_dirs::{data_home, config_home, cache_home, documents, music, downloads, pictures}` — XDG-aware, resolves `user-dirs.dirs` localized names on Unix.
- `App::new()` accepts owned `String` ids; `App::try_path()` rejects absolute and `..` paths for untrusted input.
- `migrate_dir()` / `migrate_file()` fall back to copy + remove across filesystems.
- `color_on_stdout()` / `color_on_stderr()` now return false when `TERM=dumb`.
- Add `serde` and `toml` as runtime dependencies.

## v0.1.3 · 03/08/2026

Atomic persistence and canonical app identity for the Option family. This version was made for the shared SDK on 03/08/2026 (v0.1.3).

- Add `atomic_write()` for settings and state files that must not be exposed half-written.
- Correct the shared `optionMusic` display name so SDK consumers agree on the product identity.
- Move the tempfile implementation dependency into the public library runtime.

## v0.1.2 · 01/08/2026

Docs on docs.rs, doctests from the README, and crates.io publish on version tags. This version was published to crates.io on 01/08/2026 (v0.1.2).

- Crate docs include the README (`#![doc = include_str!(…)]`) so docs.rs matches GitHub.
- CI runs `cargo test --doc` and `cargo doc`; release workflow publishes to crates.io when a `v*` tag matches `Cargo.toml`.
- Various other small tweaks

## v0.1.1 · 01/08/2026

Stderr color helper and safer env tests. This version was published to crates.io on 01/08/2026 (v0.1.1).

- `color_on_stderr()` — `NO_COLOR` plus stderr is a TTY (mirror of `color_on_stdout()`).
- Tests that mutate process environment share one crate-wide `ENV_LOCK`, so parallel threads cannot race on `$HOME` / `OPTION_HOME` / `NO_COLOR`.
- Various other small tweaks

## v0.1.0 · 01/08/2026

First crates.io release of the shared Option paths and identity helpers. This version was published to crates.io on 01/08/2026 (v0.1.0).

- App identity registry: `opsh`, `terminal`, `music`, `files`, `os`, `de`, `fat`, `notes` — mark, display name, `io.option.*` bundle id.
- Paths: `home_dir()`, `option_root()`, `expand_tilde()`, `App::dir()`, `config_toml()`, `cache_dir()`, `keys_toml()`, `session_toml()`, `path()`.
- `OPTION_HOME` override for the shared root (tests / sandboxes).
- `App::ensure()` / `App::ensure_cache()` with known legacy-tree migrate; `migrate_dir()` / `migrate_file()` helpers.
- Color: `color_enabled()` (`NO_COLOR`), `color_on_stdout()` (`NO_COLOR` + TTY).
- CI workflow (fmt, test, release build).
- Various other small tweaks
