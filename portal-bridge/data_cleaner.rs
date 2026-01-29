
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

    pub fn normalize_text(&self, text: &str) -> String {
        text.trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        self.dedupe_set.insert(normalized)
    }

    pub fn clean_dataset(&mut self, data: Vec<String>) -> Result<Vec<String>, Box<dyn Error>> {
        let mut cleaned = Vec::new();
        
        for item in data {
            if self.deduplicate(&item) {
                cleaned.push(item);
            }
        }

        if cleaned.is_empty() {
            return Err("No unique items found after cleaning".into());
        }

        Ok(cleaned)
    }

    pub fn reset(&mut self) {
        self.dedupe_set.clear();
    }

    pub fn unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        let result = cleaner.normalize_text("  HELLO World!  ");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("test"));
        assert!(!cleaner.deduplicate("TEST"));
        assert!(cleaner.deduplicate("different"));
    }

    #[test]
    fn test_clean_dataset() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "apple".to_string(),
            "APPLE".to_string(),
            "banana".to_string(),
            "apple".to_string(),
        ];
        
        let result = cleaner.clean_dataset(data).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(cleaner.unique_count(), 2);
    }
}