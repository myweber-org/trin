
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

    pub fn process_batch(&mut self, inputs: &[&str]) -> Vec<String> {
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
        
        let result1 = cleaner.process("Hello");
        let result2 = cleaner.process("hello");
        let result3 = cleaner.process("HELLO");
        
        assert_eq!(result1, Some("hello".to_string()));
        assert_eq!(result2, None);
        assert_eq!(result3, None);
        assert_eq!(cleaner.count_unique(), 1);
    }

    #[test]
    fn test_empty_input() {
        let mut cleaner = DataCleaner::new();
        
        let result1 = cleaner.process("");
        let result2 = cleaner.process("   ");
        
        assert_eq!(result1, None);
        assert_eq!(result2, None);
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        
        let inputs = vec!["Apple", "apple", "Banana", "banana", "Cherry"];
        let results = cleaner.process_batch(&inputs);
        
        assert_eq!(results.len(), 3);
        assert_eq!(cleaner.count_unique(), 3);
    }
}