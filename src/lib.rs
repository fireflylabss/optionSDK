//! Shared paths, identity, and helpers for Option family apps (`optionSDK`).
#![doc = include_str!("../README.md")]

mod app;
mod color;
mod migrate;
mod paths;

#[cfg(test)]
mod test_env;

pub use app::App;
pub use color::{color_enabled, color_on_stderr, color_on_stdout};
pub use migrate::{migrate_dir, migrate_file};
pub use paths::{expand_tilde, home_dir, option_root};
