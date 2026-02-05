
use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
    normalize_case: bool,
}

impl DataCleaner {
    pub fn new(normalize_case: bool) -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
            normalize_case,
        }
    }

    pub fn process(&mut self, input: &str) -> Result<Option<String>, Box<dyn Error>> {
        let mut processed = input.trim().to_string();

        if self.normalize_case {
            processed = processed.to_lowercase();
        }

        if processed.is_empty() {
            return Ok(None);
        }

        if self.dedupe_set.contains(&processed) {
            return Ok(None);
        }

        self.dedupe_set.insert(processed.clone());
        Ok(Some(processed))
    }

    pub fn batch_process(&mut self, inputs: &[&str]) -> Result<Vec<String>, Box<dyn Error>> {
        let mut results = Vec::new();
        
        for input in inputs {
            if let Some(processed) = self.process(input)? {
                results.push(processed);
            }
        }
        
        Ok(results)
    }

    pub fn reset(&mut self) {
        self.dedupe_set.clear();
    }

    pub fn processed_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new(false);
        let inputs = ["apple", "apple", "banana", "Apple"];
        
        let results = cleaner.batch_process(&inputs).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(cleaner.processed_count(), 3);
    }

    #[test]
    fn test_case_normalization() {
        let mut cleaner = DataCleaner::new(true);
        let inputs = ["Apple", "APPLE", "apple"];
        
        let results = cleaner.batch_process(&inputs).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "apple");
    }

    #[test]
    fn test_empty_input() {
        let mut cleaner = DataCleaner::new(false);
        let result = cleaner.process("   ").unwrap();
        assert!(result.is_none());
    }
}