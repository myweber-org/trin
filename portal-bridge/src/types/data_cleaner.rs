
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
}