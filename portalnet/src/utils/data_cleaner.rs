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

    pub fn normalize_whitespace(&mut self) {
        for record in self.records.iter_mut() {
            let parts: Vec<&str> = record.split_whitespace().collect();
            *record = parts.join(" ");
        }
    }

    pub fn to_lowercase(&mut self) {
        for record in self.records.iter_mut() {
            *record = record.to_lowercase();
        }
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("apple".to_string());
        cleaner.add_record("banana".to_string());
        cleaner.add_record("apple".to_string());
        cleaner.add_record("cherry".to_string());

        let unique = cleaner.deduplicate();
        assert_eq!(unique.len(), 3);
        assert_eq!(cleaner.get_records().len(), 3);
    }

    #[test]
    fn test_normalization() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  hello   world  ".to_string());
        cleaner.normalize_whitespace();
        
        assert_eq!(cleaner.get_records()[0], "hello world");
    }
}