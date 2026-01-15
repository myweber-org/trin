use std::collections::HashSet;
use std::error::Error;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn clean_entry(&mut self, input: &str) -> Result<Option<String>, Box<dyn Error>> {
        let trimmed = input.trim().to_lowercase();
        
        if trimmed.is_empty() {
            return Err("Empty input string".into());
        }

        if !self.is_valid_format(&trimmed) {
            return Err("Invalid format".into());
        }

        if self.dedupe_set.contains(&trimmed) {
            return Ok(None);
        }

        self.dedupe_set.insert(trimmed.clone());
        Ok(Some(trimmed))
    }

    fn is_valid_format(&self, text: &str) -> bool {
        !text.chars().any(|c| c.is_numeric()) && text.len() > 1
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }

    pub fn reset(&mut self) {
        self.dedupe_set.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_entry() {
        let mut cleaner = DataCleaner::new();
        
        let result = cleaner.clean_entry("  TEST  ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("test".to_string()));

        let duplicate = cleaner.clean_entry("test");
        assert!(duplicate.is_ok());
        assert_eq!(duplicate.unwrap(), None);
    }

    #[test]
    fn test_invalid_input() {
        let mut cleaner = DataCleaner::new();
        
        let empty = cleaner.clean_entry("");
        assert!(empty.is_err());

        let numeric = cleaner.clean_entry("abc123");
        assert!(numeric.is_err());
    }
}