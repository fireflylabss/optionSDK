use std::io::{self, IsTerminal};

/// Whether ANSI / terminal color should be used.
///
/// Returns `false` when `NO_COLOR` is set (any value), per <https://no-color.org/>.
/// Does not inspect TTY state — callers that care about pipes should use
/// [`color_on_stdout`] / [`color_on_stderr`] or combine this with their own
/// `is_terminal()` check.
pub fn color_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_none()
}

/// Whether stdout should use color: [`color_enabled`] and stdout is a terminal.
pub fn color_on_stdout() -> bool {
    color_enabled() && io::stdout().is_terminal()
}

/// Whether stderr should use color: [`color_enabled`] and stderr is a terminal.
pub fn color_on_stderr() -> bool {
    color_enabled() && io::stderr().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env;

    #[test]
    fn respects_no_color() {
        let _guard = test_env::lock();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
        assert!(color_enabled());
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
        assert!(!color_enabled());
        assert!(!color_on_stdout());
        assert!(!color_on_stderr());
        unsafe {
            std::env::remove_var("NO_COLOR");
        }
    }
}
