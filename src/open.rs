use crate::browser::Browser;
use crate::config::{Config, read_config};
use crate::dialog::Selector;
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

pub(crate) const CANCELED_ERR_MARKER: &str = "MUXIE:CANCELED";

pub(crate) fn open_url_with<O, N>(
    config: &Config,
    opener: &O,
    notifier: &N,
    selector: &dyn Selector,
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
            } // Resolve eligible browsers for this pattern (skip unknown names)
            let mut eligible: Vec<&Browser> = Vec::new();
            let mut eligible_names: Vec<String> = Vec::new();
            for name in &pat.browsers {
                if let Some(b) = by_name.get(name.as_str()) {
                    eligible.push(*b);
                    eligible_names.push(b.name.clone());
                } else if verbose >= 1 {
                    eprintln!("- Skipping unknown browser '{name}' in pattern");
                }
            }

            if eligible.is_empty() {
                continue;
            }

            // Determine attempt order, possibly via selection dialog when 2+ options exist
            let mut indices: Vec<usize> = (0..eligible.len()).collect();
            if eligible.len() >= 2 {
                let title = "Open with…";
                let redacted = crate::notify::redact_url(url);
                let message = format!("Choose a browser for: {}", redacted);
                match selector.choose(title, &message, &eligible_names, 0) {
                    Ok(Some(selected)) => {
                        // Start from selected, then wrap around the rest in order
                        let mut ordered = Vec::with_capacity(indices.len());
                        ordered.push(selected);
                        for i in (selected + 1)..indices.len() {
                            ordered.push(i);
                        }
                        for i in 0..selected {
                            ordered.push(i);
                        }
                        indices = ordered;
                    }
                    Ok(None) => {
                        // User canceled: abort operation without notifications.
                        bail!("{} Operation canceled by user", CANCELED_ERR_MARKER);
                    }
                    Err(err) => {
                        if verbose >= 1 {
                            eprintln!(
                                "Selection dialog failed ({}); proceeding without prompt",
                                err
                            );
                        }
                        // Keep indices as default order
                    }
                }
            }

            for &idx in &indices {
                let browser = eligible[idx];
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
    let selector = crate::dialog::selector_from_config(&cfg);
    open_url_with(
        &cfg,
        &opener,
        &notifier,
        selector.as_ref(),
        url,
        no_notify,
        verbose,
    )
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
            if let Some(queue) = outcomes.get_mut(&browser.name)
                && let Some(res) = queue.pop_front()
            {
                return res;
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
            dialog: crate::config::DialogOptions::default(),
        }
    }

    struct SelectIdx(pub usize);
    struct CancelSelector;
    struct ErrorSelector;
    struct NoopSelector;

    impl crate::dialog::Selector for SelectIdx {
        fn choose(
            &self,
            _title: &str,
            _message: &str,
            _options: &[String],
            _default_idx: usize,
        ) -> anyhow::Result<Option<usize>> {
            Ok(Some(self.0))
        }
    }

    impl crate::dialog::Selector for CancelSelector {
        fn choose(
            &self,
            _title: &str,
            _message: &str,
            _options: &[String],
            _default_idx: usize,
        ) -> anyhow::Result<Option<usize>> {
            Ok(None)
        }
    }

    impl crate::dialog::Selector for ErrorSelector {
        fn choose(
            &self,
            _title: &str,
            _message: &str,
            _options: &[String],
            _default_idx: usize,
        ) -> anyhow::Result<Option<usize>> {
            Err(anyhow!("boom"))
        }
    }

    impl crate::dialog::Selector for NoopSelector {
        fn choose(
            &self,
            _title: &str,
            _message: &str,
            _options: &[String],
            _default_idx: usize,
        ) -> anyhow::Result<Option<usize>> {
            Ok(None)
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
            &NoopSelector,
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
            &SelectIdx(0),
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
        let res = open_url_with(
            &cfg,
            &opener,
            &notifier,
            &NoopSelector,
            "https://example.com",
            false,
            0,
        );
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
        let res = open_url_with(
            &cfg,
            &opener,
            &notifier,
            &NoopSelector,
            "https://example.com",
            false,
            0,
        );
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
        let res = open_url_with(
            &cfg,
            &opener,
            &notifier,
            &NoopSelector,
            "https://example.com",
            false,
            0,
        );
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
        let res = open_url_with(
            &cfg,
            &opener,
            &notifier,
            &NoopSelector,
            "https://example.com",
            true,
            0,
        );
        assert!(res.is_err());
        assert!(notifier.notifications.borrow().is_empty());
    }

    #[test]
    fn empty_browsers_errors() {
        let cfg = cfg_with(vec![], vec![]);
        let opener = FakeOpener::new();
        let notifier = FakeNotifier::new();
        let res = open_url_with(
            &cfg,
            &opener,
            &notifier,
            &NoopSelector,
            "https://example.com",
            false,
            0,
        );
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
        let res = open_url_with(
            &cfg,
            &opener,
            &notifier,
            &NoopSelector,
            "https://example.com",
            false,
            0,
        );
        assert!(res.is_ok());
        assert_eq!(opener.opens.borrow().as_slice(), ["A"]);
    }

    #[test]
    fn selection_reorders_attempts_and_wraps() {
        let cfg = cfg_with(
            vec![browser("A"), browser("B"), browser("C")],
            vec![PatternEntry {
                pattern: "example.com".into(),
                browsers: vec!["A".into(), "B".into(), "C".into()],
            }],
        );
        let opener = FakeOpener::new();
        opener.queue_outcomes("B", vec![Err(anyhow!("fail B"))]);
        opener.queue_outcomes("C", vec![Ok(())]);
        let notifier = FakeNotifier::new();
        let selector = SelectIdx(1); // choose B first, then C, then A if needed
        let res = open_url_with(
            &cfg,
            &opener,
            &notifier,
            &selector,
            "https://example.com",
            false,
            0,
        );
        assert!(res.is_ok());
        assert_eq!(opener.opens.borrow().as_slice(), ["B", "C"]);
    }

    #[test]
    fn cancel_aborts_no_attempt_and_no_notify() {
        let cfg = cfg_with(
            vec![browser("A"), browser("B")],
            vec![PatternEntry {
                pattern: "example.com".into(),
                browsers: vec!["A".into(), "B".into()],
            }],
        );
        let opener = FakeOpener::new();
        let notifier = FakeNotifier::new();
        let selector = CancelSelector;
        let res = open_url_with(
            &cfg,
            &opener,
            &notifier,
            &selector,
            "https://example.com",
            false,
            0,
        );
        assert!(res.is_err());
        assert!(opener.opens.borrow().is_empty());
        assert!(notifier.notifications.borrow().is_empty());
    }

    #[test]
    fn provider_error_falls_back_to_default_order() {
        let cfg = cfg_with(
            vec![browser("A"), browser("B")],
            vec![PatternEntry {
                pattern: "example.com".into(),
                browsers: vec!["A".into(), "B".into()],
            }],
        );
        let opener = FakeOpener::new();
        opener.queue_outcomes("A", vec![Ok(())]);
        let notifier = FakeNotifier::new();
        let selector = ErrorSelector;
        let res = open_url_with(
            &cfg,
            &opener,
            &notifier,
            &selector,
            "https://example.com",
            false,
            0,
        );
        assert!(res.is_ok());
        assert_eq!(opener.opens.borrow().as_slice(), ["A"]);
    }
}
