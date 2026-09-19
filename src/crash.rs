//! Crash handler: persist a readable log when an Option app panics.
//!
//! A GUI app that dies gives the user no way to report what happened. This
//! installs a panic hook that appends a timestamped entry (version, panic
//! message, backtrace) to `~/.option/<dir>/crash.log`, once per process, so
//! the next launch can be inspected. It is best-effort: if the disk or the
//! log path is unusable we fall back to stderr rather than panic again.

use std::{
    fs::OpenOptions,
    io::{self, Write},
    panic::{self, PanicHookInfo},
    path::{Path, PathBuf},
    sync::Once,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::App;

/// How many frames of the backtrace to keep. A full Rust backtrace of a GUI
/// call chain is huge; the head is the part that names app symbols.
const MAX_FRAMES: usize = 120;

/// Install the crash hook once per process.
///
/// The hook appends a crash entry to [`crash_log_path`] and then calls the
/// previous hook, so the message still lands on stderr and the process
/// aborts with the usual nonzero status. `version` comes from the caller
/// (`env!("CARGO_PKG_VERSION")`) — the SDK does not know the app's version.
/// A second call in the same process is a no-op.
pub fn install_crash_hook(app: &App, version: &'static str) {
    static INSTALL: Once = Once::new();
    let app = app.clone();
    INSTALL.call_once(|| {
        let default = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let _ = write_crash_log(&app, version, info);
            // Keep the default hook so the message also lands on stderr and
            // the process still aborts with the usual nonzero status.
            default(info);
        }));
    });
}

/// `~/.option/<dir>/crash.log`.
pub fn crash_log_path(app: &App) -> PathBuf {
    app.dir().join("crash.log")
}

/// Capture the panic context and append one entry. Returns the path written.
fn write_crash_log(app: &App, version: &str, info: &PanicHookInfo) -> io::Result<PathBuf> {
    // Backtrace is captured now, inside the hook, so it reflects this panic
    // (RUST_BACKTRACE is not required; we always ask for one).
    let backtrace = std::backtrace::Backtrace::force_capture();
    let frames = backtrace_to_text(&backtrace);
    let payload = panic_payload(info);
    let path = crash_log_path(app);
    write_crash_entry(&path, app.display_name(), version, &payload, &frames)?;
    Ok(path)
}

/// Append one crash entry to `path`, creating the parent directory when
/// needed. This is what tests exercise — no real panic required.
fn write_crash_entry(
    path: &Path,
    display_name: &str,
    version: &str,
    payload: &str,
    frames: &str,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    let timestamp = fmt_timestamp(SystemTime::now());
    writeln!(
        file,
        "===== {display_name} crash log =====\n\
         time:    {timestamp}\n\
         version: {version}\n\
         panic:   {payload}\n\
         --- backtrace ---\n\
         {frames}\n\
         =================================\n"
    )?;
    Ok(())
}

/// The panic location (file:line) plus the message, or the raw payload.
fn panic_payload(info: &PanicHookInfo) -> String {
    let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        s.clone()
    } else {
        "non-string panic payload".to_string()
    };
    match info.location() {
        Some(loc) => format!("{loc} — {msg}"),
        None => msg,
    }
}

/// Render a `Backtrace` as a `Debug` string, capped to the first `MAX_FRAMES`
/// lines so a runaway stack does not fill the log.
fn backtrace_to_text(bt: &std::backtrace::Backtrace) -> String {
    let mut out = format!("{bt:?}");
    if let Some(idx) = out.find("\nAt ") {
        // Trim the trailing symbol-resolution note that rust adds after the
        // frames; it repeats the same addresses and bloats the log.
        out.truncate(idx);
    }
    // Keep only the head of the stack.
    let lines = out.lines().take(MAX_FRAMES).collect::<Vec<_>>();
    lines.join("\n")
}

/// Compact `YYYY-MM-DD HH:MM:SS` from a `SystemTime`.
fn fmt_timestamp(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    // days + seconds since epoch → civil date (no external crate).
    let days = secs.div_euclid(86_400);
    let secs_of_day = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    let hh = secs_of_day / 3600;
    let mm = (secs_of_day % 3600) / 60;
    let ss = secs_of_day % 60;
    format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02}")
}

/// Convert days-since-epoch to a civil (year, month, day). Howard Hinnant's
/// `civil_from_days` algorithm, originally for `date`/`std::chrono`.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_known_dates() {
        // 1970-01-01, 2026-08-24, 2000-02-29 (leap day).
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_689), (2026, 8, 24));
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
    }

    #[test]
    fn timestamp_format() {
        let s = fmt_timestamp(SystemTime::UNIX_EPOCH);
        assert_eq!(s, "1970-01-01 00:00:00");
    }

    /// Two writes append two entries; the header carries the display name,
    /// version, and payload, and a missing parent dir is created.
    #[test]
    fn crash_entries_append() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("crash.log");

        write_crash_entry(&path, "optionTerm", "1.0.0", "first boom", "frame a").unwrap();
        write_crash_entry(&path, "optionTerm", "1.0.0", "second boom", "frame b").unwrap();

        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text.matches("===== optionTerm crash log =====").count(), 2);
        assert!(text.contains("version: 1.0.0"));
        assert!(text.contains("panic:   first boom"));
        assert!(text.contains("panic:   second boom"));
        assert!(text.contains("frame a"));
        assert!(path.parent().unwrap().is_dir());
    }
}
