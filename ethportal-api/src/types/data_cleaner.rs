
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

    pub fn get_unique_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_duplicate_count(&self) -> usize {
        self.duplicates.len()
    }

    pub fn clear_duplicates(&mut self) {
        self.duplicates.clear();
    }
}

pub fn sanitize_input(input: &str) -> String {
    input
        .trim()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(cleaner.add_record("record1".to_string()));
        assert!(!cleaner.add_record("record1".to_string()));
        assert_eq!(cleaner.get_unique_count(), 1);
        assert_eq!(cleaner.get_duplicate_count(), 1);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  ".to_string());
        cleaner.add_record("valid".to_string());
        
        let valid = cleaner.validate_records();
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0], "valid");
    }

    #[test]
    fn test_sanitize() {
        let input = "  Test@123\n  ";
        let result = sanitize_input(input);
        assert_eq!(result, "Test123");
    }
}