//! Append-only debug log at %APPDATA%\ProfileRouter\router.log, size-capped.
//! The route hot path is headless, so this file is the main way to answer
//! "why did that link open in the wrong profile?".

use std::io::Write;

const MAX_BYTES: u64 = 1_000_000;

fn log_path() -> std::path::PathBuf {
    crate::config::config_dir().join("router.log")
}

fn timestamp() -> String {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::SystemInformation::GetLocalTime;
        let mut t = std::mem::zeroed();
        GetLocalTime(&mut t);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond
        )
    }
    #[cfg(not(windows))]
    {
        String::from("-")
    }
}

/// Append one line to the log. Best-effort: any failure is silently ignored so
/// logging never affects whether a link opens.
pub fn log_line(msg: &str) {
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Truncate if it has grown past the cap.
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > MAX_BYTES {
            let _ = std::fs::write(&path, b"");
        }
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{} {}", timestamp(), msg);
    }
}
