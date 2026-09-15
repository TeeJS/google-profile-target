//! Launch Chrome with a chosen profile. Uses CreateProcess directly (no shell).

use std::path::Path;
use std::process::Command;

/// Spawn `chrome.exe --profile-directory=<dir> -- <url>` detached.
/// The `--` guards against a hostile URL being parsed as a Chrome switch.
pub fn launch(chrome: &Path, profile_dir: &str, url: &str) -> std::io::Result<()> {
    Command::new(chrome)
        .arg(format!("--profile-directory={profile_dir}"))
        .arg("--")
        .arg(url)
        .spawn()?;
    Ok(())
}
