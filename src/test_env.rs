//! Serializes tests that mutate process environment variables.
//!
//! Cargo runs test threads in parallel; without a shared lock, `$HOME`,
//! `OPTION_HOME`, `NO_COLOR`, etc. race across modules.

use std::sync::{Mutex, MutexGuard};

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Hold this guard for the whole duration of any env read/write in a test.
pub(crate) fn lock() -> MutexGuard<'static, ()> {
    ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
