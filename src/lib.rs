//! Shared paths, identity, config, crash, doctor, and terminal helpers for
//! Option family apps (`optionSDK`).
#![doc = include_str!("../README.md")]

mod app;
mod color;
mod config;
mod crash;
mod fs;
mod migrate;
mod paths;
mod term;

pub mod doctor;
pub mod user_dirs;

#[cfg(test)]
mod test_env;

pub use app::App;
pub use color::{color_enabled, color_on_stderr, color_on_stdout};
pub use config::{ConfigError, load_toml, load_toml_chain, save_toml};
pub use crash::{crash_log_path, install_crash_hook};
pub use fs::atomic_write;
pub use migrate::{migrate_dir, migrate_file};
pub use paths::{expand_tilde, home_dir, option_root};
pub use term::Style;
