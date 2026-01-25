
use std::collections::HashSet;

pub struct DataCleaner {
    seen_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            seen_items: HashSet::new(),
        }
    }

    pub fn process(&mut self, input: &str) -> Option<String> {
        let normalized = input.trim().to_lowercase();
        
        if normalized.is_empty() {
            return None;
        }

        if self.seen_items.contains(&normalized) {
            return None;
        }

        self.seen_items.insert(normalized.clone());
        Some(normalized)
    }

    pub fn batch_process(&mut self, inputs: &[&str]) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.process(input))
            .collect()
    }

    pub fn reset(&mut self) {
        self.seen_items.clear();
    }

    pub fn count_unique(&self) -> usize {
        self.seen_items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_removal() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.process("hello"), Some("hello".to_string()));
        assert_eq!(cleaner.process("HELLO"), None);
        assert_eq!(cleaner.process("  hello  "), None);
    }

    #[test]
    fn test_empty_input() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.process(""), None);
        assert_eq!(cleaner.process("   "), None);
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let inputs = vec!["apple", "APPLE", "banana", "  banana  ", "cherry"];
        
        let results = cleaner.batch_process(&inputs);
        assert_eq!(results.len(), 3);
        assert_eq!(results, vec!["apple", "banana", "cherry"]);
    }

    #[test]
    fn test_reset_functionality() {
        let mut cleaner = DataCleaner::new();
        
        cleaner.process("test");
        assert_eq!(cleaner.count_unique(), 1);
        
        cleaner.reset();
        assert_eq!(cleaner.count_unique(), 0);
        
        assert_eq!(cleaner.process("test"), Some("test".to_string()));
    }
}