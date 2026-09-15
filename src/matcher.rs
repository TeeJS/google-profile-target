//! Extract the host/path from a URL and pick a profile from the config rules.

use crate::config::{Config, Rule};
use url::Url;

/// Lowercased host of an http(s)-style URL, trailing dot stripped.
/// Returns None for schemes without a host (mailto:, etc.) or unparseable input.
pub fn host_of(input: &str) -> Option<String> {
    let parsed = Url::parse(input).ok()?;
    let host = parsed.host_str()?.to_lowercase();
    Some(host.trim_end_matches('.').to_string())
}

/// Path component of a URL (e.g. "/TeeJS/repo"). "/" when there is none.
pub fn path_of(input: &str) -> Option<String> {
    Url::parse(input).ok().map(|u| u.path().to_string())
}

/// Does `pattern` match `host`? `*.example.com` matches the apex and any
/// subdomain; anything else is an exact, case-insensitive match.
pub fn matches(pattern: &str, host: &str) -> bool {
    let pattern = pattern.trim().to_lowercase();
    if let Some(suffix) = pattern.strip_prefix("*.") {
        host == suffix || host.ends_with(&format!(".{suffix}"))
    } else {
        host == pattern
    }
}

/// Does `pattern` match a URL `path`? Case-insensitive.
/// - `/TeeJS/*` matches `/TeeJS` and anything under it.
/// - `/TeeJS*`  is a plain prefix match.
/// - otherwise an exact match (trailing slash tolerated).
pub fn path_matches(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim().to_lowercase();
    let path = path.to_lowercase();
    if let Some(prefix) = pattern.strip_suffix("/*") {
        path == prefix || path.starts_with(&format!("{prefix}/"))
    } else if let Some(prefix) = pattern.strip_suffix('*') {
        path.starts_with(prefix)
    } else {
        path.trim_end_matches('/') == pattern.trim_end_matches('/')
    }
}

/// Choose a profile for `url`. Returns the profile name and the rule that
/// matched (None means the config `default` was used). A rule matches when its
/// domain matches and, if it has a `path`, the path matches too.
pub fn choose<'a>(cfg: &'a Config, url: &str) -> (&'a str, Option<&'a Rule>) {
    if let Some(host) = host_of(url) {
        let path = path_of(url).unwrap_or_default();
        for rule in &cfg.rules {
            if !matches(&rule.domain, &host) {
                continue;
            }
            if let Some(pat) = &rule.path {
                if !path_matches(pat, &path) {
                    continue;
                }
            }
            return (rule.profile.as_str(), Some(rule));
        }
    }
    (cfg.default.as_str(), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Rule};

    fn rule(domain: &str, path: Option<&str>, profile: &str) -> Rule {
        Rule {
            domain: domain.into(),
            path: path.map(Into::into),
            profile: profile.into(),
        }
    }

    fn cfg() -> Config {
        Config {
            default: "Personal".into(),
            chrome_path: None,
            rules: vec![
                rule("*.nwpipe.com", None, "Work"),
                rule("github.com", None, "Dev"),
            ],
        }
    }

    #[test]
    fn host_and_path_extraction() {
        assert_eq!(host_of("https://GitHub.com/x/y?z=1"), Some("github.com".into()));
        assert_eq!(path_of("https://github.com/TeeJS/repo?x=1"), Some("/TeeJS/repo".into()));
        assert_eq!(path_of("https://github.com"), Some("/".into()));
        assert_eq!(host_of("mailto:a@b.com"), None);
    }

    #[test]
    fn exact_and_wildcard_domain() {
        assert!(matches("github.com", "github.com"));
        assert!(!matches("github.com", "gist.github.com"));
        assert!(matches("*.nwpipe.com", "nwpipe.com"));
        assert!(matches("*.nwpipe.com", "mail.nwpipe.com"));
        assert!(!matches("*.nwpipe.com", "nwpipe.com.evil.com"));
    }

    #[test]
    fn path_matching() {
        assert!(path_matches("/TeeJS/*", "/TeeJS"));
        assert!(path_matches("/TeeJS/*", "/TeeJS/repo"));
        assert!(path_matches("/teejs/*", "/TeeJS/repo")); // case-insensitive
        assert!(!path_matches("/TeeJS/*", "/TeeJSfoo"));
        assert!(!path_matches("/TeeJS/*", "/other"));
        assert!(path_matches("/exact", "/exact/"));
    }

    #[test]
    fn first_rule_and_default() {
        let c = cfg();
        assert_eq!(choose(&c, "https://mail.nwpipe.com/x").0, "Work");
        assert_eq!(choose(&c, "https://github.com/x").0, "Dev");
        assert_eq!(choose(&c, "https://example.org").0, "Personal");
        assert_eq!(choose(&c, "mailto:a@b.com").0, "Personal");
    }

    #[test]
    fn path_rule_routing() {
        let c = Config {
            default: "Personal".into(),
            chrome_path: None,
            rules: vec![
                rule("github.com", Some("/TeeJS/*"), "Person 1"),
                rule("github.com", Some("/tschmitznwp/*"), "Springville 2nd"),
                rule("github.com", None, "Personal"), // fallback for bare github links
            ],
        };
        assert_eq!(choose(&c, "https://github.com/TeeJS/x").0, "Person 1");
        assert_eq!(choose(&c, "https://github.com/tschmitznwp/y").0, "Springville 2nd");
        assert_eq!(choose(&c, "https://github.com/").0, "Personal");
        assert_eq!(choose(&c, "https://github.com/someoneelse/z").0, "Personal");
    }
}
