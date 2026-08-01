# Changelog

We follow [Semantic Versioning](https://semver.org/) and [Keep a Changelog](https://keepachangelog.com/). optionSDK is the shared Rust crate on [crates.io](https://crates.io/crates/optionSDK) (`option_sdk` in Rust).

<details>
<summary>To see more about versioning, expand this.</summary>

Every version string starts with `v` (required), e.g. `v0.1.1`.

This project publishes **plain SemVer** to crates.io — no app surfaces, no `m` mixed tags, and no alpha/beta/stable channel suffixes in the tag. `Cargo.toml`, git tags, and crates.io use the same numeric version (`0.1.1` / `v0.1.1`).

Each release heading is the version and date (`## v0.1.2 · 01/08/2026`); under it, a short summary ends with a plain sentence like: “This version was published to crates.io on 01/08/2026 (v0.1.2).”

</details>

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
