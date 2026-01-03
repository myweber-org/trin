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

    pub fn add_record(&mut self, record: &str) -> bool {
        let trimmed = record.trim().to_string();
        
        if trimmed.is_empty() {
            return false;
        }
        
        if self.seen.contains(&trimmed) {
            return false;
        }
        
        self.seen.insert(trimmed.clone());
        self.records.push(trimmed);
        true
    }

    pub fn validate_records(&self) -> Vec<&String> {
        self.records
            .iter()
            .filter(|record| record.len() > 3 && record.len() < 256)
            .collect()
    }

    pub fn deduplicate(&mut self) -> usize {
        let original_len = self.records.len();
        let unique_records: Vec<String> = self
            .records
            .iter()
            .cloned()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        
        let removed = original_len - unique_records.len();
        self.records = unique_records;
        self.seen = self.records.iter().cloned().collect();
        
        removed
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
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
        cleaner.add_record("test");
        cleaner.add_record("test");
        cleaner.add_record("data");
        
        assert_eq!(cleaner.deduplicate(), 1);
        assert_eq!(cleaner.get_records().len(), 2);
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("abc");
        cleaner.add_record("valid_record");
        cleaner.add_record("x");
        
        let valid = cleaner.validate_records();
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0], "valid_record");
    }
}