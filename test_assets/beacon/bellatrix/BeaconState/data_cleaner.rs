use std::collections::HashSet;

pub struct DataCleaner {
    entries: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            entries: HashSet::new(),
        }
    }

    pub fn add_entry(&mut self, entry: &str) -> bool {
        let normalized = Self::normalize_string(entry);
        self.entries.insert(normalized)
    }

    pub fn get_unique_entries(&self) -> Vec<String> {
        let mut result: Vec<String> = self.entries.iter().cloned().collect();
        result.sort();
        result
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    fn normalize_string(input: &str) -> String {
        input.trim().to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_handling() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_entry("Hello"));
        assert!(!cleaner.add_entry("  HELLO  "));
        assert_eq!(cleaner.entry_count(), 1);
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_entry("  TEST  ");
        cleaner.add_entry("Test");
        cleaner.add_entry("TEST");
        assert_eq!(cleaner.entry_count(), 1);
    }

    #[test]
    fn test_unique_entries_order() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_entry("zebra");
        cleaner.add_entry("apple");
        cleaner.add_entry("banana");
        
        let entries = cleaner.get_unique_entries();
        assert_eq!(entries, vec!["apple", "banana", "zebra"]);
    }
}