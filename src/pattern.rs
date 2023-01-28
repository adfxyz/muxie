pub trait Pattern {
    fn matches(&self, url: &str) -> bool;
}

impl Pattern for String {
    fn matches(&self, url: &str) -> bool {
        if self.contains('*') {
            let s = self.replace('?', r"\?");
            let pat = wildflower::Pattern::new(&s);
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
    assert!(!"*.example.com/path?foo"
        .to_string()
        .matches("https://www.example.com/path/foo"));
}
