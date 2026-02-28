use std::collections::HashSet;

pub struct DataCleaner {
    deduplication_cache: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            deduplication_cache: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input
            .trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.deduplication_cache.contains(&normalized) {
            false
        } else {
            self.deduplication_cache.insert(normalized);
            true
        }
    }

    pub fn batch_process(&mut self, items: Vec<&str>) -> Vec<String> {
        items
            .into_iter()
            .filter(|item| self.deduplicate(item))
            .map(|item| self.normalize_text(item))
            .collect()
    }

    pub fn clear_cache(&mut self) {
        self.deduplication_cache.clear();
    }

    pub fn get_processed_count(&self) -> usize {
        self.deduplication_cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World!  "), "hello world");
        assert_eq!(cleaner.normalize_text("Data@123"), "data123");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("Hello"));
        assert!(!cleaner.deduplicate("hello"));
        assert!(!cleaner.deduplicate("  HELLO  "));
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let input = vec!["Apple", "apple", "Banana", "  banana  "];
        let result = cleaner.batch_process(input);
        assert_eq!(result.len(), 2);
        assert_eq!(result, vec!["apple", "banana"]);
    }
}