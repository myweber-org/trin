
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
            .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_dataset<T: AsRef<str>>(&mut self, data: &[T]) -> Result<Vec<String>, Box<dyn Error>> {
        let mut cleaned = Vec::new();
        
        for item in data {
            let normalized = self.normalize_text(item.as_ref());
            if self.dedupe_set.insert(normalized.clone()) {
                cleaned.push(normalized);
            }
        }
        
        if cleaned.is_empty() {
            Err("No unique items found after cleaning".into())
        } else {
            Ok(cleaned)
        }
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
        let result = cleaner.normalize_text("  Hello, World!  ");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.deduplicate("test"));
        assert!(!cleaner.deduplicate("TEST"));
        assert!(!cleaner.deduplicate(" test "));
    }

    #[test]
    fn test_clean_dataset() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["apple", "APPLE", "banana", " Banana "];
        let result = cleaner.clean_dataset(&data).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
    }
}