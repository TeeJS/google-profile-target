//! Extract the host from a URL and pick a profile from the config rules.

use crate::config::{Config, Rule};
use url::Url;

/// Lowercased host of an http(s)-style URL, trailing dot stripped.
/// Returns None for schemes without a host (mailto:, etc.) or unparseable input.
pub fn host_of(input: &str) -> Option<String> {
    let parsed = Url::parse(input).ok()?;
    let host = parsed.host_str()?.to_lowercase();
    Some(host.trim_end_matches('.').to_string())
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

/// Choose a profile for `url`. Returns the profile name and the rule that
/// matched (None means the config `default` was used).
pub fn choose<'a>(cfg: &'a Config, url: &str) -> (&'a str, Option<&'a Rule>) {
    if let Some(host) = host_of(url) {
        for rule in &cfg.rules {
            if matches(&rule.domain, &host) {
                return (rule.profile.as_str(), Some(rule));
            }
        }
    }
    (cfg.default.as_str(), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Rule};

    fn cfg() -> Config {
        Config {
            default: "Personal".into(),
            chrome_path: None,
            rules: vec![
                Rule { domain: "*.nwpipe.com".into(), profile: "Work".into() },
                Rule { domain: "github.com".into(), profile: "Dev".into() },
            ],
        }
    }

    #[test]
    fn host_extraction() {
        assert_eq!(host_of("https://GitHub.com/x/y?z=1"), Some("github.com".into()));
        assert_eq!(host_of("http://sub.example.com."), Some("sub.example.com".into()));
        assert_eq!(host_of("mailto:a@b.com"), None);
        assert_eq!(host_of("not a url"), None);
    }

    #[test]
    fn exact_match() {
        assert!(matches("github.com", "github.com"));
        assert!(!matches("github.com", "gist.github.com"));
    }

    #[test]
    fn wildcard_matches_apex_and_sub() {
        assert!(matches("*.nwpipe.com", "nwpipe.com"));
        assert!(matches("*.nwpipe.com", "mail.nwpipe.com"));
        assert!(matches("*.nwpipe.com", "a.b.nwpipe.com"));
        assert!(!matches("*.nwpipe.com", "nwpipe.com.evil.com"));
        assert!(!matches("*.nwpipe.com", "othernwpipe.com"));
    }

    #[test]
    fn first_rule_wins() {
        let c = cfg();
        assert_eq!(choose(&c, "https://mail.nwpipe.com/x").0, "Work");
        assert_eq!(choose(&c, "https://github.com/x").0, "Dev");
    }

    #[test]
    fn no_match_uses_default() {
        let c = cfg();
        assert_eq!(choose(&c, "https://example.org").0, "Personal");
        // schemes without a host still fall back rather than erroring
        assert_eq!(choose(&c, "mailto:a@b.com").0, "Personal");
    }
}
