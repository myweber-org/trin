use std::collections::HashSet;

pub struct DataCleaner {
    records: Vec<String>,
    deduplicated: bool,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            deduplicated: false,
        }
    }

    pub fn add_record(&mut self, record: String) {
        self.records.push(record);
        self.deduplicated = false;
    }

    pub fn deduplicate(&mut self) -> usize {
        if self.deduplicated {
            return 0;
        }

        let original_len = self.records.len();
        let mut seen = HashSet::new();
        
        self.records.retain(|record| {
            let normalized = record.trim().to_lowercase();
            seen.insert(normalized)
        });

        self.deduplicated = true;
        original_len - self.records.len()
    }

    pub fn validate_records(&self) -> (usize, usize) {
        let mut valid = 0;
        let mut invalid = 0;

        for record in &self.records {
            if Self::is_valid_record(record) {
                valid += 1;
            } else {
                invalid += 1;
            }
        }

        (valid, invalid)
    }

    fn is_valid_record(record: &str) -> bool {
        !record.trim().is_empty() && record.len() <= 1000
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.deduplicated = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("test".to_string());
        cleaner.add_record("TEST".to_string());
        cleaner.add_record(" test ".to_string());
        
        let removed = cleaner.deduplicate();
        assert_eq!(removed, 2);
        assert_eq!(cleaner.get_records().len(), 1);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid".to_string());
        cleaner.add_record("".to_string());
        
        let (valid, invalid) = cleaner.validate_records();
        assert_eq!(valid, 1);
        assert_eq!(invalid, 1);
    }
}