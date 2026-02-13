
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

    pub fn normalize_whitespace(&self, text: &str) -> String {
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    pub fn trim_and_lowercase(&self, text: &str) -> String {
        text.trim().to_lowercase()
    }

    pub fn clean_data(&mut self, raw_input: &str) -> Option<String> {
        let trimmed = self.trim_and_lowercase(raw_input);
        let normalized = self.normalize_whitespace(&trimmed);
        
        self.deduplicate(&normalized)
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.deduplicate("hello"), Some("hello".to_string()));
        assert_eq!(cleaner.deduplicate("hello"), None);
        assert_eq!(cleaner.deduplicate("world"), Some("world".to_string()));
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        
        assert_eq!(
            cleaner.normalize_whitespace("  multiple   spaces   here  "),
            "multiple spaces here"
        );
        
        assert_eq!(
            cleaner.trim_and_lowercase("  MIXED Case TEXT  "),
            "mixed case text"
        );
    }

    #[test]
    fn test_clean_data_workflow() {
        let mut cleaner = DataCleaner::new();
        
        let result1 = cleaner.clean_data("  Hello   World  ");
        let result2 = cleaner.clean_data("hello world");
        let result3 = cleaner.clean_data("NEW DATA");
        
        assert_eq!(result1, Some("hello world".to_string()));
        assert_eq!(result2, None);
        assert_eq!(result3, Some("new data".to_string()));
        assert_eq!(cleaner.get_unique_count(), 2);
    }
}