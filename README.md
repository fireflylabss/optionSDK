# optionSDK

Shared paths, identity, and helpers for **Option** family apps
(`opsh`, `optionTerm`, `optionMusic`, `optionFiles`, `optionCalendar`, `optionSearch`, …).

Local-first. No network. No daemon.

## Install

```toml
optionSDK = "0.1"
```

Path dependency while developing next to the other repos:

```toml
optionSDK = { path = "../optionSDK" }
```

In Rust, the crate is imported as `option_sdk`:

```rust
use option_sdk::App;
```

## Usage

```rust
use option_sdk::{App, color_on_stderr, color_on_stdout, expand_tilde};

let app = App::OPSH;
let _cfg = app.config_toml(); // ~/.option/opsh/config.toml
let _history = app.path("history"); // ~/.option/opsh/history
// app.ensure()? and app.ensure_cache()? create dirs + migrate legacy trees

assert_eq!(app.mark(), "❯");
assert_eq!(app.bundle_id(), "io.option.opsh");
assert_eq!(expand_tilde("~/Music"), option_sdk::home_dir().join("Music"));
let _ = color_on_stdout(); // NO_COLOR + stdout is a TTY
let _ = color_on_stderr(); // NO_COLOR + stderr is a TTY
```

Typed TOML config under `~/.option/<app>/config.toml`:

```rust,no_run
use option_sdk::App;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct Settings {
    week_start: Option<String>,
}

// Creates the file with defaults on first run; reads it afterwards.
let settings = App::CAL.load_config::<Settings>()?;
App::CAL.save_config(&settings)?;
# Ok::<(), option_sdk::ConfigError>(())
```

## Features (v0.1.4)

| Area | API |
|------|-----|
| Paths | `option_root()`, `home_dir()`, `expand_tilde()`, `App::dir()`, `config_toml()`, `cache_dir()`, `keys_toml()`, `session_toml()`, `path()` |
| Ensure | `App::ensure()`, `App::ensure_cache()` — create dirs + migrate known legacy trees |
| Override | `OPTION_HOME` — replace `~/.option` (tests / sandboxes) |
| Migrate | `migrate_dir()`, `migrate_file()` for app-specific leftovers |
| Identity | id, mark (◆ ❯ ♪ ◷ ◇ ⌕), display name, `io.option.*` bundle id |
| Color | `color_enabled()` (`NO_COLOR`), `color_on_stdout()` / `color_on_stderr()` (`NO_COLOR` + `TERM`≠`dumb` + TTY) |
| Persistence | `atomic_write()` — flush and sync a sibling temporary file before replacement |
| Config | `load_toml()`, `load_toml_chain()`, `save_toml()`, `App::load_config()`, `App::save_config()` — typed TOML via `atomic_write()` |
| Crash | `install_crash_hook()`, `crash_log_path()` — append panic entries to `~/.option/<app>/crash.log` |
| Doctor | `doctor::Report` / `doctor::Check` / `doctor::checks::*` — text and JSON health reports |
| Term | `Style` — ANSI `bold`/`dim`/`ok`/`warn`/`err` + `mark_line()`; `Style::stdout()` / `stderr()` honor terminal detection |
| User dirs | `user_dirs::data_home()` / `config_home()` / `cache_home()` / `documents()` / `music()` / `downloads()` / `pictures()` — XDG-aware, `user-dirs.dirs` included |

## Versioning

See [CHANGELOG.md](CHANGELOG.md).

## Layout

```text
~/.option/          # or $OPTION_HOME
  opsh/
  terminal/
  music/
  files/
  os/
  de/
  cal/
  search/           # optionSearch (legacy `needle/` migrated once)
```

## Environment

```text
OPTION_HOME   non-empty value replaces ~/.option (absolute path recommended;
              empty value falls through to ~/.option; relative paths allowed
              but discouraged — they resolve against the process CWD)
HOME          primary home lookup; USERPROFILE is the Windows fallback;
              when neither is set, home resolves to "." (CWD-relative)
NO_COLOR      when set to any value (even empty), color output is disabled
              (see https://no-color.org/)
TERM=dumb     also disables color in color_on_stdout() / color_on_stderr()
XDG_*         XDG_DATA_HOME / XDG_CONFIG_HOME / XDG_CACHE_HOME and
              XDG_DOCUMENTS_DIR / XDG_MUSIC_DIR / XDG_DOWNLOAD_DIR /
              XDG_PICTURES_DIR: absolute values honored by user_dirs::*;
              empty or relative values are ignored; on Unix,
              $XDG_CONFIG_HOME/user-dirs.dirs supplies localized names
~             expand_tilde() expands only "~" and "~/..." via home_dir();
              "~user/..." is passed through unchanged
```

## License

Apache-2.0 — see the `LICENSE` file in the repository.
