//! Shared paths, identity, and helpers for Option family apps (`optionSDK`).
//!
//! Local-first. No network. No daemon.
//!
//! Override the shared root with `OPTION_HOME` (replaces `~/.option`) for
//! tests and sandboxes.

mod app;
mod color;
mod migrate;
mod paths;

pub use app::App;
pub use color::{color_enabled, color_on_stdout};
pub use migrate::{migrate_dir, migrate_file};
pub use paths::{expand_tilde, home_dir, option_root};
