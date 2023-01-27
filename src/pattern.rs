pub trait Pattern {
    fn matches(&self, url: &str) -> bool;
}

impl Pattern for String {
    fn matches(&self, url: &str) -> bool {
        url.contains(self)
    }
}
