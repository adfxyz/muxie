pub trait Pattern {
    fn matches(&self, url: &str) -> bool;
}

impl Pattern for String {
    fn matches(&self, url: &str) -> bool {
        if self.contains('*') {
            let pat = wildflower::Pattern::new(self);
            pat.matches(url)
        } else {
            url.contains(self)
        }
    }
}

#[test]
fn test_matching() {
    assert!("example.com"
        .to_string()
        .matches("https://www.example.com/"));
    assert!("https://*.example.com/*"
        .to_string()
        .matches("https://www.example.com/"));
}
