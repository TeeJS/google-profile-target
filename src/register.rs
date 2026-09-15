//! Register (and unregister) Profile Router as a choosable Windows browser.
//! All keys are under HKCU, so no administrator rights are needed. Windows
//! still requires the user to pick it as default in Settings (UCPD-protected).

use std::io;
use winreg::enums::*;
use winreg::RegKey;

const APP_ID: &str = "ProfileRouter";
const PROGID: &str = "ProfileRouterURL";
const APP_NAME: &str = "Profile Router";
const APP_DESC: &str = "Routes links to the matching Chrome profile by domain";

fn set_default(path: &str, value: &str) -> io::Result<()> {
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(path)?;
    key.set_value("", &value.to_string())
}

fn set_value(path: &str, name: &str, value: &str) -> io::Result<()> {
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(path)?;
    key.set_value(name, &value.to_string())
}

/// Write every key needed to appear in Settings → Default Apps as a browser.
pub fn register(exe: &str) -> io::Result<()> {
    let icon = format!("{exe},0");
    let open_cmd = format!("\"{exe}\" \"%1\"");

    // Browser client registration.
    let client = format!(r"Software\Clients\StartMenuInternet\{APP_ID}");
    set_default(&client, APP_NAME)?;
    set_default(&format!(r"{client}\DefaultIcon"), &icon)?;
    set_default(&format!(r"{client}\shell\open\command"), &format!("\"{exe}\""))?;

    // Capabilities advertised to the Default Apps UI.
    let caps = format!(r"{client}\Capabilities");
    set_value(&caps, "ApplicationName", APP_NAME)?;
    set_value(&caps, "ApplicationDescription", APP_DESC)?;
    set_value(&caps, "ApplicationIcon", &icon)?;
    set_value(&format!(r"{caps}\StartMenu"), "StartMenuInternet", APP_ID)?;
    set_value(&format!(r"{caps}\URLAssociations"), "http", PROGID)?;
    set_value(&format!(r"{caps}\URLAssociations"), "https", PROGID)?;

    // The ProgId that actually runs us with the clicked URL as %1.
    let progid = format!(r"Software\Classes\{PROGID}");
    set_default(&progid, "Profile Router URL")?;
    set_default(&format!(r"{progid}\DefaultIcon"), &icon)?;
    set_default(&format!(r"{progid}\shell\open\command"), &open_cmd)?;

    // Announce our Capabilities to Windows.
    set_value(
        r"Software\RegisteredApplications",
        APP_ID,
        &format!(r"Software\Clients\StartMenuInternet\{APP_ID}\Capabilities"),
    )?;

    Ok(())
}

/// Remove everything `register` created. Missing keys are not an error.
pub fn unregister() -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let _ = hkcu.delete_subkey_all(format!(r"Software\Clients\StartMenuInternet\{APP_ID}"));
    let _ = hkcu.delete_subkey_all(format!(r"Software\Classes\{PROGID}"));
    if let Ok(key) = hkcu.open_subkey_with_flags(r"Software\RegisteredApplications", KEY_ALL_ACCESS) {
        let _ = key.delete_value(APP_ID);
    }
    Ok(())
}
