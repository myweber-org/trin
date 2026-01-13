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

    pub fn add_record(&mut self, record: &str) {
        self.records.push(record.trim().to_string());
    }

    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.records.retain(|r| seen.insert(r.clone()));
    }

    pub fn normalize_case(&mut self) {
        for record in &mut self.records {
            *record = record.to_lowercase();
        }
    }

    pub fn sort_records(&mut self) {
        self.records.sort();
    }

    pub fn get_records(&self) -> &Vec<String> {
        &self.records
    }

    pub fn clean(&mut self) {
        self.deduplicate();
        self.normalize_case();
        self.sort_records();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaning() {
        let mut cleaner = DataCleaner::new();
        cleaner.add_record("  Apple  ");
        cleaner.add_record("banana");
        cleaner.add_record("Apple");
        cleaner.add_record("Banana");
        cleaner.add_record("apple");

        cleaner.clean();
        let result = cleaner.get_records();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"apple".to_string()));
        assert!(result.contains(&"banana".to_string()));
    }
}