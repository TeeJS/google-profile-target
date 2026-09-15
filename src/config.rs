//! Load and create the TOML rules file at %APPDATA%\ProfileRouter\config.toml.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Sample config written on first run. Kept in sync with config.example.toml.
pub const EXAMPLE: &str = r#"# Profile Router config.
# Clicked links are matched against the rules below, top to bottom; first match wins.
# `profile` is the Chrome profile's display name (run `profile-router --list-profiles`)
# or its folder name ("Profile 1", "Default"). Unmatched links use `default`.

# Fallback profile for links that match no rule. "Default" is always valid.
default = "Default"

# Optional: force a specific chrome.exe. Auto-detected if omitted.
# chrome_path = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe"

# --- Rules ---------------------------------------------------------------
# domain: exact host ("github.com") or wildcard ("*.example.com" matches the
#         apex example.com and any subdomain).

# [[rule]]
# domain = "*.nwpipe.com"
# profile = "Work"

# [[rule]]
# domain = "github.com"
# profile = "Dev"
"#;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub domain: String,
    pub profile: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Fallback profile for links that match no rule.
    pub default: String,
    /// Optional explicit path to chrome.exe.
    #[serde(default)]
    pub chrome_path: Option<String>,
    /// Ordered rules. Authored in TOML as `[[rule]]` blocks.
    #[serde(default, rename = "rule")]
    pub rules: Vec<Rule>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default: "Default".to_string(),
            chrome_path: None,
            rules: Vec::new(),
        }
    }
}

/// %APPDATA%\ProfileRouter (falls back to the current dir if APPDATA is unset).
pub fn config_dir() -> PathBuf {
    match std::env::var_os("APPDATA") {
        Some(appdata) => PathBuf::from(appdata).join("ProfileRouter"),
        None => PathBuf::from("."),
    }
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// Load config, creating a template on first run. Never fails: on any error it
/// returns a safe built-in default so a link always opens somewhere.
pub fn load() -> Config {
    let path = config_path();
    if !path.exists() {
        let _ = std::fs::create_dir_all(config_dir());
        let _ = std::fs::write(&path, EXAMPLE);
        // The template has no active rules; the embedded default matches it.
        return Config::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(text) => match toml::from_str::<Config>(&text) {
            Ok(cfg) => cfg,
            Err(e) => {
                crate::logging::log_line(&format!(
                    "config parse error ({}): {e}; using built-in default",
                    path.display()
                ));
                Config::default()
            }
        },
        Err(e) => {
            crate::logging::log_line(&format!(
                "config read error ({}): {e}; using built-in default",
                path.display()
            ));
            Config::default()
        }
    }
}
