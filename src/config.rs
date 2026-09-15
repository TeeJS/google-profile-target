//! Load and create the TOML rules file at %APPDATA%\ProfileRouter\config.toml.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Markers around the auto-maintained Chrome-profile comment block. Everything
/// between them is rewritten by `--config`; the user's rules are never touched.
const PROFILES_BEGIN: &str = "# >>> chrome profiles (auto-updated by --config) >>>";
const PROFILES_END: &str = "# <<< chrome profiles <<<";

/// Sample config written on first run. Kept in sync with config.example.toml.
pub const EXAMPLE: &str = r#"# Profile Router config.
# Clicked links are matched against the rules below, top to bottom; first match wins.
# `profile` is the Chrome profile's display name or its folder name (see the
# auto-updated list above). Unmatched links use `default`.

# Fallback profile for links that match no rule. "Default" is always valid.
default = "Default"

# Optional: force a specific chrome.exe. Auto-detected if omitted.
# chrome_path = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe"

# --- Rules ---------------------------------------------------------------
# domain: exact host ("github.com") or wildcard ("*.example.com" matches the
#         apex example.com and any subdomain).
# path:   optional. Match on the URL path too. "/TeeJS/*" matches /TeeJS and
#         anything under it. Only the clicked link's path is seen (redirects and
#         link-shorteners hide the real destination). List path rules before a
#         plain domain rule for the same host.

# [[rule]]
# domain = "*.nwpipe.com"
# profile = "Work"

# [[rule]]
# domain = "github.com"
# path = "/TeeJS/*"
# profile = "Dev"

# [[rule]]
# domain = "github.com"
# profile = "Personal"
"#;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub domain: String,
    /// Optional URL path matcher (prefix, `*` wildcard). None = match any path.
    #[serde(default)]
    pub path: Option<String>,
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
        // New file starts with a current profile list so the user can write
        // rules without running --list-profiles first.
        let _ = std::fs::write(&path, refresh_profiles_comment(EXAMPLE));
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

/// The auto-maintained comment block listing this machine's Chrome profiles.
fn build_profiles_block() -> String {
    let list = crate::chrome::profiles();
    let mut s = String::from(PROFILES_BEGIN);
    s.push('\n');
    if list.is_empty() {
        s.push_str("#   (no Chrome profiles found)\n");
    } else {
        s.push_str("#   DIRECTORY      DISPLAY NAME\n");
        for (dir, name) in list {
            s.push_str(&format!("#   {dir:<14} {name}\n"));
        }
    }
    s.push_str(PROFILES_END);
    s
}

/// Return `text` with the profile block inserted at the top or, if the markers
/// already exist, replaced in place. Rules and other content are untouched.
pub fn refresh_profiles_comment(text: &str) -> String {
    let block = build_profiles_block();
    match (text.find(PROFILES_BEGIN), text.find(PROFILES_END)) {
        (Some(start), Some(end_start)) if end_start >= start => {
            let end = end_start + PROFILES_END.len();
            format!("{}{}{}", &text[..start], block, &text[end..])
        }
        _ => format!("{block}\n\n{text}"),
    }
}

/// Rewrite the profile block in the on-disk config (best-effort, non-destructive).
pub fn refresh_profiles_in_file() {
    let path = config_path();
    if let Ok(text) = std::fs::read_to_string(&path) {
        let updated = refresh_profiles_comment(&text);
        if updated != text {
            let _ = std::fs::write(&path, updated);
        }
    }
}
