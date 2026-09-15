# Profile Router

Open clicked links in the **Chrome profile that matches the link's domain** —
automatically, from Outlook or any app.

Windows only lets one program handle `http`/`https`, and Chrome can't route by
domain on its own. Profile Router is a tiny native "browser shim": you set it as
your default browser once, and from then on every clicked link is handed to it.
It reads the link's domain, looks up a rule, and relaunches Chrome with the right
`--profile-directory`. Links that match no rule open in a configurable fallback
profile, so a link never just disappears.

## How it works
1. Windows sends the clicked URL to `profile-router.exe`.
2. It reads `%APPDATA%\ProfileRouter\config.toml` and picks a profile by domain.
3. It maps the friendly profile name to Chrome's folder name (via Chrome's
   `Local State`) and runs `chrome.exe --profile-directory=<dir> -- <url>`.

No admin rights, no background service, no network listener.

## Build
Requires the Rust toolchain (MSVC). Install once:

```bash
winget install --id Rustlang.Rustup -e
```

Then:

```bash
cargo build --release
```

The binary is `target\release\profile-router.exe`. Put it somewhere stable
(e.g. `%LOCALAPPDATA%\ProfileRouter\`) before registering — the install path is
baked into the registration.

## Set up
```bash
# 1. See your Chrome profiles so you know what to put in the rules file.
profile-router.exe --list-profiles

# 2. Edit the rules (a starter file is created on first run).
profile-router.exe --config

# 3. Register as a browser, then set it as default when Settings opens.
profile-router.exe --register
```

Step 3 opens **Settings → Default apps**. Set **Profile Router** as the default
for **HTTP** and **HTTPS**. Windows requires this to be done by hand — no app can
set itself as default (UCPD protection).

## Configure
`%APPDATA%\ProfileRouter\config.toml` (TOML, comments allowed). See
[`config.example.toml`](config.example.toml). Running `--config` refreshes an
auto-maintained comment block at the top of the file listing this machine's
Chrome profiles, so you never have to run `--list-profiles` to write a rule.

```toml
default = "Personal"        # fallback profile for unmatched links

[[rule]]
domain = "*.nwpipe.com"     # apex + any subdomain
profile = "Work"

# Optional path matching — route two GitHub orgs to different profiles:
[[rule]]
domain = "github.com"
path = "/TeeJS/*"           # matches /TeeJS and anything under it
profile = "Dev"

[[rule]]
domain = "github.com"       # catch-all for other github.com links
profile = "Personal"
```

`profile` may be a Chrome display name ("Work") or a folder name ("Profile 1",
"Default"). Rules are checked top to bottom; the first match wins.

- **domain**: exact host or `*.` wildcard (apex + subdomains).
- **path** (optional): `/prefix/*` or `/prefix*`. Only the *clicked link's* path
  is seen — redirects and link-shorteners hide the real destination. List path
  rules before a plain domain rule for the same host.

## Commands
| Command | Does |
| --- | --- |
| `profile-router <url>` | Route a URL (what Windows calls on a link click) |
| `profile-router --register` | Register as a browser, then open Default Apps |
| `profile-router --unregister` | Remove the browser registration |
| `profile-router --list-profiles` | List Chrome profiles |
| `profile-router --dry-run <url>` | Show which profile a URL would use (no launch) |
| `profile-router --config` | Print and reveal the config file |

## Troubleshooting
Every routed click is logged to `%APPDATA%\ProfileRouter\router.log` — url,
matched rule, chosen profile, resolved directory, and whether Chrome launched.
Start there when a link opens in the wrong profile.
