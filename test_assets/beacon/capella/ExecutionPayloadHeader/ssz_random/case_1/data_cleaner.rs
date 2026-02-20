
use std::collections::HashSet;

pub struct DataCleaner {
    unique_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            unique_items: HashSet::new(),
        }
    }

    pub fn add_item(&mut self, item: &str) -> bool {
        let normalized = Self::normalize_string(item);
        self.unique_items.insert(normalized)
    }

    pub fn get_unique_items(&self) -> Vec<String> {
        let mut items: Vec<String> = self.unique_items.iter().cloned().collect();
        items.sort();
        items
    }

    pub fn clear(&mut self) {
        self.unique_items.clear();
    }

    pub fn count(&self) -> usize {
        self.unique_items.len()
    }

    fn normalize_string(s: &str) -> String {
        s.trim().to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_duplicate_items() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_item("Apple"));
        assert!(!cleaner.add_item("apple"));
        assert!(!cleaner.add_item("  APPLE  "));
        assert_eq!(cleaner.count(), 1);
    }

    #[test]
    fn test_get_sorted_items() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_item("Zebra");
        cleaner.add_item("apple");
        cleaner.add_item("Banana");
        
        let items = cleaner.get_unique_items();
        assert_eq!(items, vec!["apple", "banana", "zebra"]);
    }

    #[test]
    fn test_clear_functionality() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_item("Test");
        assert_eq!(cleaner.count(), 1);
        
        cleaner.clear();
        assert_eq!(cleaner.count(), 0);
    }
}