//! Shared `doctor` report (text and JSON) for Option family apps.
//!
//! Apps build a [`Report`], push [`Check`]s (by hand or via [`checks`]),
//! then render with [`Report::render_text`] or [`Report::render_json`].
//! JSON is emitted by a fixed, hand-rolled writer — no `serde_json`.

use crate::App;
use crate::term::Style;

/// Outcome of a single doctor check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok,
    Warn,
    Fail,
}

impl Status {
    /// Text rendering uses the wording opsh already prints.
    fn text_word(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Fail => "missing",
        }
    }

    fn json_word(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
    }
}

/// One labeled check with an optional detail (path, resolved binary, …).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub label: String,
    pub status: Status,
    pub detail: Option<String>,
}

/// A list of checks for one app, renderable as text or JSON.
#[derive(Debug, Clone)]
pub struct Report {
    app: App,
    checks: Vec<Check>,
}

impl Report {
    pub fn new(app: &App) -> Self {
        Self {
            app: app.clone(),
            checks: Vec::new(),
        }
    }

    /// Append a check; returns `self` for chaining.
    pub fn push(&mut self, check: Check) -> &mut Self {
        self.checks.push(check);
        self
    }

    /// Append an [`Status::Ok`] check; returns `self` for chaining.
    pub fn ok(&mut self, label: impl Into<String>, detail: Option<String>) -> &mut Self {
        self.push(Check {
            label: label.into(),
            status: Status::Ok,
            detail,
        })
    }

    /// Append a [`Status::Warn`] check; returns `self` for chaining.
    pub fn warn(&mut self, label: impl Into<String>, detail: Option<String>) -> &mut Self {
        self.push(Check {
            label: label.into(),
            status: Status::Warn,
            detail,
        })
    }

    /// Append a [`Status::Fail`] check; returns `self` for chaining.
    pub fn fail(&mut self, label: impl Into<String>, detail: Option<String>) -> &mut Self {
        self.push(Check {
            label: label.into(),
            status: Status::Fail,
            detail,
        })
    }

    pub fn checks(&self) -> &[Check] {
        &self.checks
    }

    /// No check failed (warnings still count as healthy).
    pub fn is_healthy(&self) -> bool {
        !self.checks.iter().any(|c| c.status == Status::Fail)
    }

    /// `<mark> <id> doctor` header plus one `  label: detail (status)` line
    /// per check, ending with a newline.
    pub fn render_text(&self, style: &Style) -> String {
        let mut out = style.mark_line(&self.app, &format!("{} doctor", self.app.id()));
        out.push('\n');
        for check in &self.checks {
            let word = match check.status {
                Status::Ok => style.ok(check.status.text_word()),
                Status::Warn => style.warn(check.status.text_word()),
                Status::Fail => style.err(check.status.text_word()),
            };
            match &check.detail {
                Some(detail) => {
                    out.push_str(&format!("  {}: {detail} ({word})\n", check.label));
                }
                None => {
                    out.push_str(&format!("  {}: {word}\n", check.label));
                }
            }
        }
        out
    }

    /// Single line: `{"app":"<id>","ok":<bool>,"checks":[…]}`.
    pub fn render_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{\"app\":");
        out.push_str(&json_string(self.app.id()));
        out.push_str(",\"ok\":");
        out.push_str(if self.is_healthy() { "true" } else { "false" });
        out.push_str(",\"checks\":[");
        for (i, check) in self.checks.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("{\"label\":");
            out.push_str(&json_string(&check.label));
            out.push_str(",\"status\":\"");
            out.push_str(check.status.json_word());
            out.push_str("\",\"detail\":");
            match &check.detail {
                Some(detail) => out.push_str(&json_string(detail)),
                None => out.push_str("null"),
            }
            out.push('}');
        }
        out.push_str("]}");
        out
    }
}

/// Escape a string for embedding in a JSON value.
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Ready-made checks that most apps repeat.
pub mod checks {
    use std::path::Path;

    use super::{Check, Status};
    use crate::App;

    /// Whether `app.dir()` exists; detail is the path.
    pub fn state_dir(app: &App) -> Check {
        let dir = app.dir();
        Check {
            label: "state dir".to_string(),
            status: if dir.is_dir() {
                Status::Ok
            } else {
                Status::Fail
            },
            detail: Some(dir.display().to_string()),
        }
    }

    /// Whether `path` is a file; detail is the path.
    pub fn file(label: &str, path: &Path) -> Check {
        Check {
            label: label.to_string(),
            status: if path.is_file() {
                Status::Ok
            } else {
                Status::Fail
            },
            detail: Some(path.display().to_string()),
        }
    }

    /// Whether `name` resolves on `PATH` (also tries `name.exe` on Windows);
    /// detail is the resolved path, or the bare name when missing.
    pub fn binary(label: &str, name: &str) -> Check {
        #[cfg(windows)]
        let names = [name.to_string(), format!("{name}.exe")];
        #[cfg(not(windows))]
        let names = [name.to_string()];

        let resolved = std::env::var_os("PATH").and_then(|paths| {
            std::env::split_paths(&paths)
                .find_map(|dir| names.iter().map(|n| dir.join(n)).find(|p| p.is_file()))
        });
        Check {
            label: label.to_string(),
            status: if resolved.is_some() {
                Status::Ok
            } else {
                Status::Fail
            },
            detail: Some(
                resolved
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| name.to_string()),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env;

    #[test]
    fn json_escapes() {
        assert_eq!(json_string("a\"b"), "\"a\\\"b\"");
        assert_eq!(json_string("a\\b"), "\"a\\\\b\"");
        assert_eq!(json_string("a\nb"), "\"a\\nb\"");
        assert_eq!(json_string("a\u{0007}b"), "\"a\\u0007b\"");
        assert_eq!(json_string(""), "\"\"");
    }

    #[test]
    fn is_healthy_tracks_fail() {
        let mut report = Report::new(&App::OPSH);
        report.warn("w", None);
        assert!(report.is_healthy());
        report.fail("f", Some("gone".to_string()));
        assert!(!report.is_healthy());
    }

    #[test]
    fn render_text_plain() {
        let mut report = Report::new(&App::OPSH);
        report
            .ok("state dir", Some("/tmp/x".to_string()))
            .warn("shell", None)
            .fail("binary", Some("zig".to_string()));
        let text = report.render_text(&Style::plain());
        assert!(!text.contains('\x1b'));
        assert!(text.starts_with("❯ opsh doctor\n"));
        assert!(text.contains("  state dir: /tmp/x (ok)\n"));
        assert!(text.contains("  shell: warn\n"));
        assert!(text.contains("  binary: zig (missing)\n"));
        assert!(text.ends_with('\n'));
    }

    #[test]
    fn render_json_shape() {
        let mut report = Report::new(&App::OPSH);
        report
            .ok("state dir", Some("/tmp/x".to_string()))
            .fail("bin\"ary", None);
        assert_eq!(
            report.render_json(),
            "{\"app\":\"opsh\",\"ok\":false,\"checks\":[\
             {\"label\":\"state dir\",\"status\":\"ok\",\"detail\":\"/tmp/x\"},\
             {\"label\":\"bin\\\"ary\",\"status\":\"fail\",\"detail\":null}]}"
        );
    }

    #[test]
    fn state_dir_check() {
        let _guard = test_env::lock();
        let root = tempfile::tempdir().unwrap();
        // SAFETY: tests serialize env mutation via ENV_LOCK.
        unsafe {
            std::env::set_var("OPTION_HOME", root.path());
        }
        let check = checks::state_dir(&App::FILES);
        assert_eq!(check.status, Status::Fail);
        assert_eq!(check.label, "state dir");
        App::FILES.ensure().unwrap();
        assert_eq!(checks::state_dir(&App::FILES).status, Status::Ok);
        unsafe {
            std::env::remove_var("OPTION_HOME");
        }
    }

    #[test]
    fn file_check() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("rc");
        assert_eq!(checks::file("rc", &path).status, Status::Fail);
        std::fs::write(&path, "x").unwrap();
        assert_eq!(checks::file("rc", &path).status, Status::Ok);
    }

    #[cfg(unix)]
    #[test]
    fn binary_check_finds_sh() {
        let check = checks::binary("shell", "sh");
        assert_eq!(check.status, Status::Ok);
        assert_eq!(
            checks::binary("nope", "option-sdk-no-such-bin").status,
            Status::Fail
        );
    }
}
