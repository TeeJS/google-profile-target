# Project Charter — Profile Router

## What is the one thing this must do?
Clicked links (from Outlook or any app) open in the Chrome profile that matches
the link's domain, automatically.

## What would be wrong if we shipped "working" software without it?
A clicked link must **always** open somewhere. If no rule matches, it opens in a
configured fallback profile — a link must never silently vanish.

## What is explicitly off-limits as a workaround?
- No spoofing the Windows `UserChoice` hash or disabling the UCPD driver to force
  ourselves to be the default browser. Setting default is the user's one-time
  manual step in Settings.
- No hand-hacking the registry to store routing rules. Rules live in a
  hand-editable TOML config file.
- No shells, no listening sockets, no packed/UPX binaries (EDR-sensitive machine).

## What is the deployment target and the backup location?
- **Target:** managed Windows 11 box.
- **Backup:** this git repo (https://github.com/TeeJS/google-profile-target).

## How will we verify it is done?
`cargo test` passes; `--list-profiles` shows real profiles; `--dry-run` picks the
right profile per rules; after `--register` + setting default, clicking a link in
Outlook opens it in the matching profile (confirmed via `router.log`). Release
binary is code-signed with Azure Trusted Signing before real deployment.

## Design summary
Tiny Rust "browser shim" (`profile-router.exe`), GUI subsystem (no console flash
on the hot path), registers under HKCU as a choosable browser, reads Chrome's
`Local State` to map friendly profile names to `--profile-directory` folder
names, launches `chrome.exe --profile-directory=<dir> -- <url>` via CreateProcess.
