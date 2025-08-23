pub trait Pattern {
    fn matches(&self, url: &str) -> bool;
}

impl Pattern for String {
    /// Check if a URL matches a pattern.
    /// Pattern may contain `*` wildcard that matches any number of characters.
    /// If there is no `*` in the pattern, it is treated as a substring.
    ///
    /// Examples:
    /// - example.com matches https://www.example.com/any/path
    /// - example.com also matches https://domain.com?redirect=https://www.example.com
    /// - https://*.example.com/*/index.html matches https://www.example.com/any/path/index.html
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
    assert!(
        "example.com"
            .to_string()
            .matches("https://www.example.com/")
    );
    assert!(
        "example.com"
            .to_string()
            .matches("https://domain.com?redirect=https://www.example.com")
    );
    assert!(
        "https://*.example.com/*"
            .to_string()
            .matches("https://www.example.com/")
    );
    assert!(
        !"https://*.example.com/*"
            .to_string()
            .matches("https://domain.com?redirect=https://www.example.com")
    );
    assert!(
        !"*.example.com/path?foo"
            .to_string()
            .matches("https://www.example.com/path/foo")
    );
}
