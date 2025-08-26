use crate::browser::Browser;
use crate::config::{Config, read_config};
use crate::notify::{DefaultNotifier, Notifier, NotifyPrefs};
use crate::pattern::Pattern;
use anyhow::{Context, Result, bail};

pub(crate) trait UrlOpener {
    fn open(&self, browser: &Browser, url: &str) -> Result<()>;
}

pub(crate) struct DefaultOpener;

impl UrlOpener for DefaultOpener {
    fn open(&self, browser: &Browser, url: &str) -> Result<()> {
        let mut command = std::process::Command::new(&browser.executable);
        let mut url_arg_found = false;
        for arg in &browser.args {
            match arg.as_str() {
                "%u" | "%U" => {
                    url_arg_found = true;
                    command.arg(url);
                }
                _ => {
                    command.arg(arg);
                }
            }
        }
        if !url_arg_found {
            command.arg(url);
        }
        command.spawn()?;
        Ok(())
    }
}

pub(crate) fn open_url_with<O, N>(
    config: &Config,
    opener: &O,
    notifier: &N,
    url: &str,
    no_notify: bool,
    verbose: u8,
) -> Result<()>
where
    O: UrlOpener,
    N: Notifier,
{
    if config.browsers.is_empty() {
        bail!("No browsers configured. Run 'muxie install' to set up the browsers.");
    }
    let notify_prefs = NotifyPrefs {
        enabled: config.notifications.enabled && !no_notify,
        redact_urls: config.notifications.redact_urls,
    };
    // Build a lookup map for browsers by name (preserve order separately)
    let mut by_name: std::collections::HashMap<&str, &Browser> = std::collections::HashMap::new();
    for b in &config.browsers {
        by_name.insert(b.name.as_str(), b);
    }

    for pat in &config.patterns {
        if pat.browsers.is_empty() {
            continue; // ignored pattern per PRD
        }
        if pat.pattern.matches(url) {
            if verbose >= 1 {
                eprintln!("Pattern '{}' matched", pat.pattern);
            }
            for name in &pat.browsers {
                let browser = match by_name.get(name.as_str()) {
                    Some(b) => *b,
                    None => {
                        if verbose >= 1 {
                            eprintln!("- Skipping unknown browser '{name}' in pattern");
                        }
                        continue;
                    }
                };
                if verbose >= 1 {
                    eprintln!("- Trying browser '{}'", browser.name);
                }
                match opener.open(browser, url) {
                    Ok(_) => return Ok(()),
                    Err(err) => {
                        eprintln!(
                            "Warning: Failed to open URL '{}' with browser '{}': {}",
                            url, browser.name, err
                        );
                        eprintln!("Trying next browser...");
                        notifier.notify_error(
                            url,
                            pat.pattern.as_str(),
                            &browser.name,
                            &format!("{err}"),
                            &notify_prefs,
                        );
                        continue;
                    }
                }
            }
        }
    }
    let default_browser = &config.browsers[0];
    if verbose >= 1 {
        eprintln!(
            "No patterns matched, using default browser '{}'",
            default_browser.name
        );
    }
    let result = opener.open(default_browser, url).with_context(|| {
        format!(
            "Failed to open URL '{}' with default browser '{}'",
            url, default_browser.name
        )
    });

    if let Err(err) = &result {
        notifier.notify_error(
            url,
            "default",
            default_browser.name.as_str(),
            &format!("{err}"),
            &notify_prefs,
        );
    }
    result
}

pub(crate) fn open_url(url: &str, no_notify: bool, verbose: u8) -> Result<()> {
    let cfg = read_config()?;
    let opener = DefaultOpener;
    let notifier = DefaultNotifier;
    open_url_with(&cfg, &opener, &notifier, url, no_notify, verbose)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, PatternEntry};
    use anyhow::{Result, anyhow};
    use std::cell::RefCell;
    use std::collections::{HashMap, VecDeque};

    #[derive(Clone, Default)]
    struct TestPrefsCapture {
        enabled: bool,
    }

    struct FakeOpener {
        outcomes: RefCell<HashMap<String, VecDeque<Result<()>>>>,
        opens: RefCell<Vec<String>>, // order of browser names attempted
    }

    impl FakeOpener {
        fn new() -> Self {
            Self {
                outcomes: RefCell::new(HashMap::new()),
                opens: RefCell::new(Vec::new()),
            }
        }
        fn queue_outcomes(&self, name: &str, outcomes: Vec<Result<()>>) {
            self.outcomes
                .borrow_mut()
                .insert(name.to_string(), outcomes.into_iter().collect());
        }
    }

    impl UrlOpener for FakeOpener {
        fn open(&self, browser: &Browser, _url: &str) -> Result<()> {
            self.opens.borrow_mut().push(browser.name.clone());
            let mut outcomes = self.outcomes.borrow_mut();
            if let Some(queue) = outcomes.get_mut(&browser.name) {
                if let Some(res) = queue.pop_front() {
                    return res;
                }
            }
            Ok(())
        }
    }

    struct FakeNotifier {
        notifications: RefCell<Vec<(String, String, String, TestPrefsCapture)>>, // (url, rule, browser, prefs)
    }

    impl FakeNotifier {
        fn new() -> Self {
            Self {
                notifications: RefCell::new(Vec::new()),
            }
        }
    }

    impl Notifier for FakeNotifier {
        fn notify_error(
            &self,
            url: &str,
            rule: &str,
            browser: &str,
            error_summary: &str,
            prefs: &NotifyPrefs,
        ) {
            if !prefs.enabled {
                return;
            }
            self.notifications.borrow_mut().push((
                url.to_string(),
                rule.to_string(),
                browser.to_string(),
                TestPrefsCapture {
                    enabled: prefs.enabled,
                },
            ));
            let _ = error_summary;
        }
    }

    fn browser(name: &str) -> Browser {
        Browser {
            name: name.to_string(),
            executable: name.to_lowercase(),
            args: vec!["%u".to_string()],
        }
    }

    fn cfg_with(browsers: Vec<Browser>, patterns: Vec<PatternEntry>) -> Config {
        Config {
            version: 1,
            browsers,
            patterns,
            notifications: crate::config::Notifications::default(),
        }
    }

    #[test]
    fn success_on_first_match() {
        let cfg = cfg_with(
            vec![browser("A"), browser("B")],
            vec![PatternEntry {
                pattern: "example.com".into(),
                browsers: vec!["A".into()],
            }],
        );
        let opener = FakeOpener::new();
        opener.queue_outcomes("A", vec![Ok(())]);
        let notifier = FakeNotifier::new();
        let res = open_url_with(
            &cfg,
            &opener,
            &notifier,
            "https://www.example.com",
            false,
            0,
        );
        assert!(res.is_ok());
        assert_eq!(opener.opens.borrow().as_slice(), ["A"]);
        assert!(notifier.notifications.borrow().is_empty());
    }

    #[test]
    fn retry_on_failure_then_success() {
        let cfg = cfg_with(
            vec![browser("A"), browser("B")],
            vec![PatternEntry {
                pattern: "example.com".into(),
                browsers: vec!["A".into(), "B".into()],
            }],
        );
        let opener = FakeOpener::new();
        opener.queue_outcomes("A", vec![Err(anyhow!("fail A"))]);
        opener.queue_outcomes("B", vec![Ok(())]);
        let notifier = FakeNotifier::new();
        let res = open_url_with(
            &cfg,
            &opener,
            &notifier,
            "https://www.example.com/x",
            false,
            0,
        );
        assert!(res.is_ok());
        assert_eq!(opener.opens.borrow().as_slice(), ["A", "B"]);
        let notifies = notifier.notifications.borrow();
        assert_eq!(notifies.len(), 1);
        let (_url, rule, browser, _prefs) = &notifies[0];
        assert_eq!(rule, "example.com");
        assert_eq!(browser, "A");
    }

    #[test]
    fn no_match_uses_default() {
        let cfg = cfg_with(
            vec![browser("A"), browser("B")],
            vec![PatternEntry {
                pattern: "nope".into(),
                browsers: vec!["B".into()],
            }],
        );
        let opener = FakeOpener::new();
        opener.queue_outcomes("A", vec![Ok(())]);
        let notifier = FakeNotifier::new();
        let res = open_url_with(&cfg, &opener, &notifier, "https://example.com", false, 0);
        assert!(res.is_ok());
        assert_eq!(opener.opens.borrow().as_slice(), ["A"]);
        assert!(notifier.notifications.borrow().is_empty());
    }

    #[test]
    fn all_fail_with_match_triggers_notify_with_rule() {
        let cfg = cfg_with(
            vec![browser("A")],
            vec![PatternEntry {
                pattern: "example.com".into(),
                browsers: vec!["A".into()],
            }],
        );
        let opener = FakeOpener::new();
        opener.queue_outcomes("A", vec![Err(anyhow!("first")), Err(anyhow!("default"))]);
        let notifier = FakeNotifier::new();
        let res = open_url_with(&cfg, &opener, &notifier, "https://example.com", false, 0);
        assert!(res.is_err());
        // Tried match then default (same browser index 0 twice)
        assert_eq!(opener.opens.borrow().as_slice(), ["A", "A"]);
        let notifies = notifier.notifications.borrow();
        assert_eq!(notifies.len(), 2);
        // First notify from match failure
        let (_url1, rule1, browser1, prefs1) = &notifies[0];
        assert_eq!(rule1, "example.com");
        assert_eq!(browser1, "A");
        assert!(prefs1.enabled);
        // Second notify from default failure
        let (_url2, rule2, browser2, _prefs2) = &notifies[1];
        assert_eq!(rule2, "default");
        assert_eq!(browser2, "A");
    }

    #[test]
    fn all_fail_no_match_triggers_notify_default_rule() {
        let cfg = cfg_with(
            vec![browser("A")],
            vec![PatternEntry {
                pattern: "nope".into(),
                browsers: vec!["A".into()],
            }],
        );
        let opener = FakeOpener::new();
        opener.queue_outcomes("A", vec![Err(anyhow!("default"))]);
        let notifier = FakeNotifier::new();
        let res = open_url_with(&cfg, &opener, &notifier, "https://example.com", false, 0);
        assert!(res.is_err());
        assert_eq!(opener.opens.borrow().as_slice(), ["A"]);
        let notifies = notifier.notifications.borrow();
        assert_eq!(notifies.len(), 1);
        let (_url, rule, browser, _prefs) = &notifies[0];
        assert_eq!(rule, "default");
        assert_eq!(browser, "A");
    }

    #[test]
    fn no_notify_flag_suppresses_notifications() {
        let cfg = cfg_with(
            vec![browser("A")],
            vec![PatternEntry {
                pattern: "nope".into(),
                browsers: vec!["A".into()],
            }],
        );
        let opener = FakeOpener::new();
        opener.queue_outcomes("A", vec![Err(anyhow!("default"))]);
        let notifier = FakeNotifier::new();
        let res = open_url_with(&cfg, &opener, &notifier, "https://example.com", true, 0);
        assert!(res.is_err());
        assert!(notifier.notifications.borrow().is_empty());
    }

    #[test]
    fn empty_browsers_errors() {
        let cfg = cfg_with(vec![], vec![]);
        let opener = FakeOpener::new();
        let notifier = FakeNotifier::new();
        let res = open_url_with(&cfg, &opener, &notifier, "https://example.com", false, 0);
        assert!(res.is_err());
        assert!(opener.opens.borrow().is_empty());
        assert!(notifier.notifications.borrow().is_empty());
    }

    #[test]
    fn unknown_browser_is_skipped_and_fallback_applies() {
        let cfg = cfg_with(
            vec![browser("A")],
            vec![PatternEntry {
                pattern: "example.com".into(),
                browsers: vec!["Missing".into(), "A".into()],
            }],
        );
        let opener = FakeOpener::new();
        let notifier = FakeNotifier::new();
        let res = open_url_with(&cfg, &opener, &notifier, "https://example.com", false, 0);
        assert!(res.is_ok());
        assert_eq!(opener.opens.borrow().as_slice(), ["A"]);
    }
}
