
use std::collections::HashSet;
use std::error::Error;

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

    pub fn add_record(&mut self, record: &str) {
        self.records.push(record.trim().to_string());
        self.deduplicated = false;
    }

    pub fn deduplicate(&mut self) -> usize {
        let mut unique_set = HashSet::new();
        let mut deduped_records = Vec::new();
        
        for record in &self.records {
            if unique_set.insert(record.clone()) {
                deduped_records.push(record.clone());
            }
        }
        
        let removed_count = self.records.len() - deduped_records.len();
        self.records = deduped_records;
        self.deduplicated = true;
        
        removed_count
    }

    pub fn validate_records(&self) -> Result<(), Box<dyn Error>> {
        for (i, record) in self.records.iter().enumerate() {
            if record.is_empty() {
                return Err(format!("Empty record at index {}", i).into());
            }
            if record.len() > 1000 {
                return Err(format!("Record too long at index {}", i).into());
            }
            if !record.chars().all(|c| c.is_ascii() && !c.is_control()) {
                return Err(format!("Invalid characters in record at index {}", i).into());
            }
        }
        Ok(())
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn is_deduplicated(&self) -> bool {
        self.deduplicated
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
        cleaner.add_record("test");
        cleaner.add_record("test");
        cleaner.add_record("unique");
        
        let removed = cleaner.deduplicate();
        assert_eq!(removed, 1);
        assert_eq!(cleaner.get_records().len(), 2);
        assert!(cleaner.is_deduplicated());
    }

    #[test]
    fn test_validation() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("valid record");
        
        assert!(cleaner.validate_records().is_ok());
        
        cleaner.add_record("");
        assert!(cleaner.validate_records().is_err());
    }
}