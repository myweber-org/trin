
use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn deduplicate(&mut self, input: &str) -> Option<String> {
        if self.dedupe_set.contains(input) {
            None
        } else {
            self.dedupe_set.insert(input.to_string());
            Some(input.to_string())
        }
    }

    pub fn normalize_whitespace(text: &str) -> String {
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    pub fn trim_and_lowercase(text: &str) -> String {
        text.trim().to_lowercase()
    }

    pub fn clean_data(&mut self, raw_data: &str) -> Option<String> {
        let normalized = Self::normalize_whitespace(raw_data);
        let cleaned = Self::trim_and_lowercase(&normalized);
        self.deduplicate(&cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert_eq!(cleaner.deduplicate("test"), Some("test".to_string()));
        assert_eq!(cleaner.deduplicate("test"), None);
    }

    #[test]
    fn test_normalization() {
        let input = "  Hello   World  ";
        assert_eq!(DataCleaner::normalize_whitespace(input), "Hello World");
        assert_eq!(DataCleaner::trim_and_lowercase(input), "hello   world");
    }

    #[test]
    fn test_full_clean() {
        let mut cleaner = DataCleaner::new();
        let result = cleaner.clean_data("  Apple   Banana  ");
        assert_eq!(result, Some("apple banana".to_string()));
        
        let duplicate = cleaner.clean_data("  apple   banana  ");
        assert_eq!(duplicate, None);
    }
}