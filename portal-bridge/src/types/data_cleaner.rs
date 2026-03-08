
use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
    duplicates: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            duplicates: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: String) -> bool {
        if self.duplicates.contains(&record) {
            return false;
        }
        
        if self.records.contains(&record) {
            self.duplicates.insert(record.clone());
            return false;
        }
        
        self.records.push(record);
        true
    }

    pub fn validate_records(&self) -> Vec<&String> {
        self.records
            .iter()
            .filter(|record| !record.trim().is_empty())
            .collect()
    }

    pub fn get_unique_records(&self) -> Vec<String> {
        self.records.clone()
    }

    pub fn remove_duplicates(&mut self) -> usize {
        let original_count = self.records.len();
        self.records.retain(|record| !self.duplicates.contains(record));
        original_count - self.records.len()
    }

    pub fn clear_all(&mut self) {
        self.records.clear();
        self.duplicates.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_record("record1".to_string()));
        assert!(!cleaner.add_record("record1".to_string()));
        assert_eq!(cleaner.get_unique_records().len(), 1);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  ".to_string());
        cleaner.add_record("valid".to_string());
        assert_eq!(cleaner.validate_records().len(), 1);
    }
}use std::collections::HashSet;
use std::hash::Hash;

pub fn remove_duplicates<T: Eq + Hash + Clone>(items: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    
    for item in items {
        if !seen.contains(item) {
            seen.insert(item.clone());
            result.push(item.clone());
        }
    }
    
    result
}

pub fn normalize_strings(strings: &[String]) -> Vec<String> {
    strings
        .iter()
        .map(|s| s.trim().to_lowercase())
        .collect()
}

pub fn clean_numeric_data(numbers: &[f64]) -> Vec<f64> {
    numbers
        .iter()
        .filter(|&&n| n.is_finite() && n >= 0.0)
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let data = vec![1, 2, 2, 3, 4, 4, 5];
        let cleaned = remove_duplicates(&data);
        assert_eq!(cleaned, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_normalize_strings() {
        let strings = vec![
            "  HELLO  ".to_string(),
            "World".to_string(),
            "  TEST  ".to_string(),
        ];
        let normalized = normalize_strings(&strings);
        assert_eq!(normalized, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_clean_numeric_data() {
        let numbers = vec![1.0, f64::NAN, -5.0, 10.0, f64::INFINITY, 0.0];
        let cleaned = clean_numeric_data(&numbers);
        assert_eq!(cleaned, vec![1.0, 10.0, 0.0]);
    }
}