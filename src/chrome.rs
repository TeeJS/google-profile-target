//! Locate chrome.exe and map friendly profile names to on-disk directory names.

use crate::config::Config;
use std::path::PathBuf;

/// %LOCALAPPDATA%\Google\Chrome\User Data
fn user_data_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(|p| PathBuf::from(p).join("Google").join("Chrome").join("User Data"))
}

/// (directory name, display name) for every profile, read from Local State.
/// e.g. ("Profile 1", "Work"). Empty vec if the file is missing/unreadable.
pub fn profiles() -> Vec<(String, String)> {
    let Some(path) = user_data_dir().map(|d| d.join("Local State")) else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if let Some(cache) = json
        .get("profile")
        .and_then(|p| p.get("info_cache"))
        .and_then(|c| c.as_object())
    {
        for (dir, info) in cache {
            let display = info
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or(dir)
                .to_string();
            out.push((dir.clone(), display));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Resolve a config `profile` value (a display name or a literal dir name) to
/// the `--profile-directory` value Chrome expects. None if it can't be matched.
pub fn resolve_profile_dir(name: &str) -> Option<String> {
    let list = profiles();
    // Exact directory name wins first (lets users pin "Profile 1" / "Default").
    if list.iter().any(|(dir, _)| dir.eq_ignore_ascii_case(name)) {
        return Some(name.to_string());
    }
    // Otherwise match on display name, case-insensitively.
    list.iter()
        .find(|(_, display)| display.eq_ignore_ascii_case(name))
        .map(|(dir, _)| dir.clone())
}

/// Find chrome.exe: config override, then common install locations, then the
/// App Paths registry entry.
pub fn find_chrome(cfg: &Config) -> Option<PathBuf> {
    if let Some(p) = &cfg.chrome_path {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }

    let rel = "Google\\Chrome\\Application\\chrome.exe";
    for var in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        if let Some(base) = std::env::var_os(var) {
            let path = PathBuf::from(base).join(rel);
            if path.exists() {
                return Some(path);
            }
        }
    }

    // App Paths default value (HKCU then HKLM).
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;
    let sub = r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\chrome.exe";
    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        if let Ok(key) = RegKey::predef(root).open_subkey(sub) {
            if let Ok(val) = key.get_value::<String, _>("") {
                let path = PathBuf::from(val.trim_matches('"'));
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }
    None
}
