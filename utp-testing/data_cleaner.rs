
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

    pub fn process_string(&mut self, input: &str) -> Option<String> {
        let normalized = input.trim().to_lowercase();
        
        if normalized.is_empty() {
            return None;
        }

        if self.unique_items.insert(normalized.clone()) {
            Some(normalized)
        } else {
            None
        }
    }

    pub fn process_batch(&mut self, inputs: &[&str]) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.process_string(input))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.unique_items.len()
    }

    pub fn clear(&mut self) {
        self.unique_items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_removal() {
        let mut cleaner = DataCleaner::new();
        
        let result1 = cleaner.process_string("Hello");
        let result2 = cleaner.process_string("hello");
        let result3 = cleaner.process_string("HELLO");
        
        assert!(result1.is_some());
        assert!(result2.is_none());
        assert!(result3.is_none());
        assert_eq!(cleaner.get_unique_count(), 1);
    }

    #[test]
    fn test_empty_string() {
        let mut cleaner = DataCleaner::new();
        
        let result = cleaner.process_string("   ");
        
        assert!(result.is_none());
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        
        let inputs = vec!["apple", "Apple", "banana", "BANANA", "cherry"];
        let results = cleaner.process_batch(&inputs);
        
        assert_eq!(results.len(), 3);
        assert_eq!(cleaner.get_unique_count(), 3);
    }
}