use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
    }

    pub fn deduplicate(&mut self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut unique_records = Vec::new();

        for record in self.records.drain(..) {
            if seen.insert(record.clone()) {
                unique_records.push(record);
            }
        }

        self.records = unique_records.clone();
        unique_records
    }

    pub fn validate_records(&self) -> (usize, usize) {
        let total = self.records.len();
        let valid = self.records
            .iter()
            .filter(|record| !record.trim().is_empty())
            .count();

        (total, valid)
    }

    pub fn get_statistics(&self) -> (usize, usize) {
        let (total, valid) = self.validate_records();
        let duplicates = total - self.records.len();
        
        (valid, duplicates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test1".to_string());
        cleaner.add_record("test2".to_string());
        cleaner.add_record("test1".to_string());
        
        let unique = cleaner.deduplicate();
        assert_eq!(unique.len(), 2);
        assert_eq!(cleaner.records.len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        cleaner.add_record("  ".to_string());
        
        let (total, valid) = cleaner.validate_records();
        assert_eq!(total, 3);
        assert_eq!(valid, 1);
    }
}