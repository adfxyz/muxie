pub struct NotifyPrefs {
    pub enabled: bool,
    pub redact_urls: bool,
}

pub(crate) fn redact_url(url: &str) -> String {
    // Show only host if possible; fallback to original URL
    let trimmed = url.trim();
    let start = trimmed.find("://").map(|i| i + 3).unwrap_or(0);
    let rest = &trimmed[start..];
    // Cut at first '/', '?', or '#'
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let host = &rest[..end];
    if host.is_empty() {
        trimmed.to_string()
    } else {
        host.to_string()
    }
}

pub(crate) fn notify_error(
    url: &str,
    rule: &str,
    browser: &str,
    error_summary: &str,
    prefs: &NotifyPrefs,
) {
    if !prefs.enabled {
        return;
    }
    let shown_url = if prefs.redact_urls {
        redact_url(url)
    } else {
        url.to_string()
    };
    let title = "Muxie: Failed to open";
    let body = format!("{shown_url} via rule '{rule}' → {browser}: {error_summary}");
    // Best-effort notification; swallow all errors
    let _ = notify_rust::Notification::new()
        .summary(title)
        .body(&body)
        .appname("Muxie")
        .icon("muxie")
        .show();
}

// Dependency trait for notifications and a default impl.
pub(crate) trait Notifier {
    fn notify_error(
        &self,
        url: &str,
        rule: &str,
        browser: &str,
        error_summary: &str,
        prefs: &NotifyPrefs,
    );
}

#[derive(Default, Clone, Copy)]
pub(crate) struct DefaultNotifier;

impl Notifier for DefaultNotifier {
    fn notify_error(
        &self,
        url: &str,
        rule: &str,
        browser: &str,
        error_summary: &str,
        prefs: &NotifyPrefs,
    ) {
        // Delegate to the module function for actual delivery
        notify_error(url, rule, browser, error_summary, prefs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redaction_extracts_host() {
        assert_eq!(redact_url("https://example.com/path?q=1"), "example.com");
        assert_eq!(redact_url("http://www.example.com"), "www.example.com");
        assert_eq!(redact_url("example.com/path"), "example.com");
        assert_eq!(
            redact_url("   https://sub.example.com#frag   "),
            "sub.example.com"
        );
        // Fallback when no host
        assert_eq!(redact_url("://"), "://");
    }

    // No rate limiter test as rate limiting is removed.
}
