use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
    seen: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            seen: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: String) -> bool {
        if self.seen.contains(&record) {
            return false;
        }
        
        if Self::validate_record(&record) {
            self.seen.insert(record.clone());
            self.records.push(record);
            true
        } else {
            false
        }
    }

    fn validate_record(record: &str) -> bool {
        !record.trim().is_empty() && record.len() <= 1000
    }

    pub fn get_clean_records(&self) -> &[String] {
        &self.records
    }

    pub fn remove_duplicates(&mut self) -> usize {
        let original_count = self.records.len();
        let mut unique_records = Vec::new();
        let mut new_seen = HashSet::new();

        for record in self.records.drain(..) {
            if !new_seen.contains(&record) {
                new_seen.insert(record.clone());
                unique_records.push(record);
            }
        }

        self.records = unique_records;
        self.seen = new_seen;
        original_count - self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.seen.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("test".to_string());
        assert_eq!(cleaner.get_clean_records().len(), 1);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        assert!(!cleaner.add_record("".to_string()));
        assert!(cleaner.add_record("valid data".to_string()));
    }
}