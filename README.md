# optionSDK

Shared paths, identity, and helpers for **Option** family apps
(`opsh`, `optionTerm`, `optionMusic`, `optionFiles`, `optionNotes`, …).

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
let dir = app.ensure()?;                 // ~/.option/opsh (+ legacy migrate)
let cfg = app.config_toml();             // ~/.option/opsh/config.toml
let cache = app.ensure_cache()?;         // ~/.option/opsh/cache
let history = app.path("history");       // ~/.option/opsh/history

assert_eq!(app.mark(), "◆");
assert_eq!(app.bundle_id(), "io.option.opsh");
assert_eq!(expand_tilde("~/Music"), option_sdk::home_dir().join("Music"));
let _ = color_on_stdout();               // NO_COLOR + stdout is a TTY
let _ = color_on_stderr();               // NO_COLOR + stderr is a TTY
```

## Features (v0.1)

| Area | API |
|------|-----|
| Paths | `option_root()`, `home_dir()`, `expand_tilde()`, `App::dir()`, `config_toml()`, `cache_dir()`, `keys_toml()`, `session_toml()`, `path()` |
| Ensure | `App::ensure()`, `App::ensure_cache()` — create dirs + migrate known legacy trees |
| Override | `OPTION_HOME` — replace `~/.option` (tests / sandboxes) |
| Migrate | `migrate_dir()`, `migrate_file()` for app-specific leftovers |
| Identity | id, mark (◇◆♪), display name, `io.option.*` bundle id |
| Color | `color_enabled()` (`NO_COLOR`), `color_on_stdout()` / `color_on_stderr()` (`NO_COLOR` + TTY) |

## Versioning

See [VERSIONING.md](VERSIONING.md) and [CHANGELOG.md](CHANGELOG.md).

## Layout

```text
~/.option/          # or $OPTION_HOME
  opsh/
  terminal/
  music/
  files/
  os/
  de/
  notes/
```

## License

Apache-2.0 — see [`LICENSE`](LICENSE).
