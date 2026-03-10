use std::collections::HashSet;

pub struct DataCleaner {
    dedupe_set: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            dedupe_set: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<&str>) -> Vec<String> {
        data.iter()
            .filter(|&&item| self.deduplicate(item))
            .map(|&item| self.normalize_text(item))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.dedupe_set.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let data = vec!["Apple", "apple", "APPLE", "Banana", "banana"];
        let cleaned = cleaner.clean_dataset(data);
        
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
        assert!(cleaned.contains(&"apple".to_string()));
        assert!(cleaned.contains(&"banana".to_string()));
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  HELLO World  "), "hello world");
    }
}use std::collections::HashSet;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: String,
    pub category: String,
}

pub struct DataCleaner {
    records: Vec<DataRecord>,
    seen_ids: HashSet<u32>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            records: Vec::new(),
            seen_ids: HashSet::new(),
        }
    }

    pub fn add_record(&mut self, record: DataRecord) -> Result<(), Box<dyn Error>> {
        if record.value.is_empty() {
            return Err("Value cannot be empty".into());
        }

        if record.category.is_empty() {
            return Err("Category cannot be empty".into());
        }

        if self.seen_ids.contains(&record.id) {
            return Err("Duplicate ID found".into());
        }

        self.seen_ids.insert(record.id);
        self.records.push(record);
        Ok(())
    }

    pub fn deduplicate_by_value(&mut self) {
        let mut unique_values = HashSet::new();
        self.records.retain(|record| unique_values.insert(record.value.clone()));
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn get_statistics(&self) -> (usize, usize) {
        let total = self.records.len();
        let categories: HashSet<_> = self.records.iter().map(|r| &r.category).collect();
        (total, categories.len())
    }

    pub fn export_records(&self) -> Vec<DataRecord> {
        self.records.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_valid_record() {
        let mut cleaner = DataCleaner::new();
        let record = DataRecord {
            id: 1,
            value: "test".to_string(),
            category: "A".to_string(),
        };

        assert!(cleaner.add_record(record).is_ok());
        assert_eq!(cleaner.get_statistics().0, 1);
    }

    #[test]
    fn test_duplicate_id_rejection() {
        let mut cleaner = DataCleaner::new();
        let record1 = DataRecord {
            id: 1,
            value: "test1".to_string(),
            category: "A".to_string(),
        };
        let record2 = DataRecord {
            id: 1,
            value: "test2".to_string(),
            category: "B".to_string(),
        };

        assert!(cleaner.add_record(record1).is_ok());
        assert!(cleaner.add_record(record2).is_err());
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        let records = vec![
            DataRecord {
                id: 1,
                value: "duplicate".to_string(),
                category: "A".to_string(),
            },
            DataRecord {
                id: 2,
                value: "duplicate".to_string(),
                category: "B".to_string(),
            },
            DataRecord {
                id: 3,
                value: "unique".to_string(),
                category: "C".to_string(),
            },
        ];

        for record in records {
            let _ = cleaner.add_record(record);
        }

        cleaner.deduplicate_by_value();
        assert_eq!(cleaner.get_statistics().0, 2);
    }
}