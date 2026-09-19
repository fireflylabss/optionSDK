//! Minimal ANSI styling for terminal output.
//!
//! [`Style::stdout`] / [`Style::stderr`] pick up color support from
//! [`crate::color_on_stdout`] / [`crate::color_on_stderr`] (`NO_COLOR`,
//! `TERM=dumb`, and TTY detection). When color is off every method returns
//! the input unchanged, so output is always safe to print.

use crate::App;

const RESET: &str = "\x1b[0m";

/// Whether ANSI color is enabled, plus the escape helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    color: bool,
}

impl Style {
    /// Color when stdout is a terminal and `NO_COLOR` / `TERM=dumb` are unset.
    pub fn stdout() -> Self {
        Self::new(crate::color_on_stdout())
    }

    /// Color when stderr is a terminal and `NO_COLOR` / `TERM=dumb` are unset.
    pub fn stderr() -> Self {
        Self::new(crate::color_on_stderr())
    }

    /// Never emit ANSI escapes.
    pub fn plain() -> Self {
        Self::new(false)
    }

    pub fn new(color: bool) -> Self {
        Self { color }
    }

    /// Whether this style emits ANSI escapes.
    pub fn enabled(&self) -> bool {
        self.color
    }

    pub fn bold(&self, s: &str) -> String {
        self.wrap("\x1b[1m", s)
    }

    pub fn dim(&self, s: &str) -> String {
        self.wrap("\x1b[2m", s)
    }

    /// Green — success.
    pub fn ok(&self, s: &str) -> String {
        self.wrap("\x1b[32m", s)
    }

    /// Yellow — warning.
    pub fn warn(&self, s: &str) -> String {
        self.wrap("\x1b[33m", s)
    }

    /// Red — error.
    pub fn err(&self, s: &str) -> String {
        self.wrap("\x1b[31m", s)
    }

    /// `<mark> <text>` with the app mark in bold, e.g. `◆ opsh doctor`.
    pub fn mark_line(&self, app: &App, text: &str) -> String {
        format!("{} {text}", self.bold(app.mark()))
    }

    fn wrap(&self, code: &str, s: &str) -> String {
        if self.color {
            format!("{code}{s}{RESET}")
        } else {
            s.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_has_no_escapes() {
        let style = Style::plain();
        assert!(!style.enabled());
        assert_eq!(style.bold("x"), "x");
        assert_eq!(style.dim("x"), "x");
        assert_eq!(style.ok("x"), "x");
        assert_eq!(style.warn("x"), "x");
        assert_eq!(style.err("x"), "x");
        assert!(!style.bold("x").contains('\x1b'));
    }

    #[test]
    fn wraps_with_ansi() {
        let style = Style::new(true);
        assert!(style.enabled());
        assert_eq!(style.bold("x"), "\x1b[1mx\x1b[0m");
        assert_eq!(style.dim("x"), "\x1b[2mx\x1b[0m");
        assert_eq!(style.ok("x"), "\x1b[32mx\x1b[0m");
        assert_eq!(style.warn("x"), "\x1b[33mx\x1b[0m");
        assert_eq!(style.err("x"), "\x1b[31mx\x1b[0m");
    }

    #[test]
    fn mark_line_plain() {
        let style = Style::plain();
        assert_eq!(style.mark_line(&App::OPSH, "opsh doctor"), "◆ opsh doctor");
    }

    #[test]
    fn mark_line_color() {
        let style = Style::new(true);
        assert_eq!(
            style.mark_line(&App::OPSH, "doctor"),
            "\x1b[1m◆\x1b[0m doctor"
        );
    }
}
