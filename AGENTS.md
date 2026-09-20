# AGENTS.md — optionSDK contract for coding agents

Import with `use option_sdk::App;` (crate name is `option_sdk`).

Modules: TOML config (`load_toml`, `save_toml`, `load_toml_chain`,
`App::load_config`, `App::save_config`), crash hook (`install_crash_hook`,
`crash_log_path`), `doctor` reports (`Report`, `Check`, `checks::*`),
terminal `Style`, and `user_dirs::*` (XDG folders).

## Apps (8)

`opsh`, `terminal`, `music`, `files`, `os`, `de`, `cal`,
`search` (see `App::ALL`). `App::new(id, mark, display_name)` builds a
custom app (`dir_name` = `id`, no legacy migration). `App::known(id)`
looks up by id (`"needle"` resolves to `SEARCH`).

## Paths

- `option_root()` → `$OPTION_HOME` or `~/.option`
- `home_dir()` → `$HOME`, then `$USERPROFILE`, else `.`
- `expand_tilde()` → expands only `~` and `~/...`; `~user/...` passes through
- `App::dir()` → `~/.option/<dir>/`; `config_toml()`, `cache_dir()`,
  `keys_toml()`, `session_toml()`, `path(rel)` join under it

## Env

- `OPTION_HOME`: non-empty value overrides root (absolute recommended;
  empty falls through; relative is CWD-relative, avoid in prod)
- `HOME` / `USERPROFILE`: home lookup order; unset both → `.`
- `NO_COLOR`: any value (even empty) disables color (`color_enabled()`,
  `color_on_stdout()`, `color_on_stderr()`)
- `TERM=dumb` also disables color in `color_on_stdout()` / `color_on_stderr()`
- `XDG_DATA_HOME` / `XDG_CONFIG_HOME` / `XDG_CACHE_HOME` / `XDG_*_DIR`:
  absolute values honored by `user_dirs::*`; empty or relative ignored;
  Unix also reads `$XDG_CONFIG_HOME/user-dirs.dirs`

## Rules

- Persist TOML/state only via `atomic_write()` — never raw `fs::write`.
  Typed config goes through `save_toml()` / `App::save_config()` (both wrap
  `atomic_write()`); load with `load_toml()` / `App::load_config()` and
  `load_toml_chain()` for legacy path fallbacks.
- `App::ensure()` creates the dir and migrates known legacy trees once;
  `ensure_cache()` also creates `cache/`. `migrate_dir()`/`migrate_file()`
  cover app-specific leftovers.
- `App::path()` takes relative paths only — no absolute segments, no `..`.
- GUI apps call `install_crash_hook(&app, env!("CARGO_PKG_VERSION"))`
  before any toolkit init; the log lands at `crash_log_path(&app)`.
- User folders via `user_dirs::*` — never hardcode `~/Documents`,
  `~/.local/share`, etc.
- `doctor [--json]` commands build a `doctor::Report` and render it with
  `render_text(&Style::stdout())` / `render_json()`.

## Verify

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Tests that touch env must hold `crate::test_env::lock()` and use a
`tempfile::tempdir()` as `OPTION_HOME` (see `src/app.rs` ensure test).
MSRV is 1.85 — do not use newer-only std/rustc features.
