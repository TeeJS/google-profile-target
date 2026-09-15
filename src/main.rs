// GUI subsystem: the route hot path must never flash a console window when a
// link is clicked. CLI commands reattach to the parent console for output.
// Disabled under `cfg(test)` so `cargo test` output stays visible.
#![cfg_attr(not(test), windows_subsystem = "windows")]

mod chrome;
mod config;
mod launch;
mod logging;
mod matcher;
mod register;

use std::io::Write;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let first = args.first().map(String::as_str).unwrap_or("");

    // Route mode: the first argument is a URL (anything not starting with --).
    if !first.is_empty() && !first.starts_with("--") {
        route(first);
        return;
    }

    // Management mode: make output visible if launched from a terminal.
    attach_console();
    match first {
        "--register" => cmd_register(),
        "--unregister" => cmd_unregister(),
        "--list-profiles" => cmd_list_profiles(),
        "--dry-run" => cmd_dry_run(args.get(1).map(String::as_str)),
        "--config" => cmd_config(),
        _ => print_help(),
    }
}

/// The hot path: pick a profile for `url` and hand it to Chrome.
fn route(url: &str) {
    let cfg = config::load();
    let (profile, rule) = matcher::choose(&cfg, url);
    let dir = chrome::resolve_profile_dir(profile).unwrap_or_else(|| profile.to_string());

    match chrome::find_chrome(&cfg) {
        Some(chrome_exe) => {
            let result = launch::launch(&chrome_exe, &dir, url);
            logging::log_line(&format!(
                "url={url} rule={} profile={profile} dir={dir} chrome=\"{}\" spawn={}",
                rule.map(|r| r.domain.as_str()).unwrap_or("<default>"),
                chrome_exe.display(),
                if result.is_ok() { "ok" } else { "FAILED" },
            ));
            if let Err(e) = result {
                logging::log_line(&format!("spawn error: {e}"));
            }
        }
        None => logging::log_line(&format!(
            "url={url} profile={profile} ERROR: chrome.exe not found"
        )),
    }
}

fn out(line: &str) {
    let _ = writeln!(std::io::stdout(), "{line}");
}

fn exe_path() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "profile-router.exe".to_string())
}

fn cmd_register() {
    let exe = exe_path();
    match register::register(&exe) {
        Ok(()) => {
            out(&format!("Registered \"Profile Router\" ({exe}) as an available browser."));
            out("");
            out("Next step (required, one time):");
            out("  Settings > Apps > Default apps > set Profile Router as default for HTTP and HTTPS.");
            out("Opening Default Apps settings now...");
            // explorer.exe opens the settings URI without spawning a shell.
            let _ = Command::new("explorer").arg("ms-settings:defaultapps").spawn();
        }
        Err(e) => out(&format!("Registration failed: {e}")),
    }
}

fn cmd_unregister() {
    match register::unregister() {
        Ok(()) => out("Unregistered Profile Router. Set another browser as default in Settings."),
        Err(e) => out(&format!("Unregister failed: {e}")),
    }
}

fn cmd_list_profiles() {
    let profiles = chrome::profiles();
    if profiles.is_empty() {
        out("No Chrome profiles found (is Chrome installed for this user?).");
        return;
    }
    out("Chrome profiles (use either column as a rule's `profile` value):");
    out(&format!("  {:<14} {}", "DIRECTORY", "DISPLAY NAME"));
    for (dir, name) in profiles {
        out(&format!("  {dir:<14} {name}"));
    }
}

fn cmd_dry_run(url: Option<&str>) {
    let Some(url) = url else {
        out("Usage: profile-router --dry-run \"https://example.com/...\"");
        return;
    };
    let cfg = config::load();
    let (profile, rule) = matcher::choose(&cfg, url);
    let dir = chrome::resolve_profile_dir(profile);
    out(&format!("URL      : {url}"));
    out(&format!("Host     : {}", matcher::host_of(url).unwrap_or_else(|| "<none>".into())));
    out(&format!(
        "Matched  : {}",
        rule.map(|r| r.domain.as_str()).unwrap_or("<no rule - using default>")
    ));
    out(&format!("Profile  : {profile}"));
    out(&format!(
        "Directory: {}",
        dir.unwrap_or_else(|| format!("{profile} (unresolved - will pass through to Chrome)"))
    ));
}

fn cmd_config() {
    let path = config::config_path();
    // Ensure it exists so there is something to reveal.
    let _ = config::load();
    out(&format!("Config file: {}", path.display()));
    let _ = Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn();
}

fn print_help() {
    out("Profile Router - open clicked links in the matching Chrome profile.");
    out("");
    out("Usage:");
    out("  profile-router <url>            Route a URL (used by Windows on link click)");
    out("  profile-router --register       Register as a browser, then open Default Apps");
    out("  profile-router --unregister     Remove the browser registration");
    out("  profile-router --list-profiles  Show Chrome profiles (for writing rules)");
    out("  profile-router --dry-run <url>  Show which profile a URL would use");
    out("  profile-router --config         Show and reveal the config file");
}

/// Attach to the parent console (if any) so CLI output is visible when run from
/// a terminal. Harmless no-op when double-clicked.
fn attach_console() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}
