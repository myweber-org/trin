
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

    pub fn normalize_string(&self, input: &str) -> String {
        input.trim().to_lowercase()
    }

    pub fn deduplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_string(item);
        if self.dedupe_set.contains(&normalized) {
            false
        } else {
            self.dedupe_set.insert(normalized);
            true
        }
    }

    pub fn clean_data(&mut self, data: Vec<&str>) -> Vec<String> {
        let mut cleaned = Vec::new();
        
        for item in data {
            if self.deduplicate(item) {
                cleaned.push(self.normalize_string(item));
            }
        }
        
        cleaned
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
        
        let cleaned = cleaner.clean_data(data);
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_string("  TEST String  "), "test string");
    }
}use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_numeric_field(value: f64, threshold: f64) -> Option<f64> {
    if value.is_finite() && value.abs() <= threshold {
        Some(value)
    } else {
        None
    }
}

fn validate_category(category: &str, valid_categories: &[&str]) -> bool {
    valid_categories.contains(&category)
}

pub fn process_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(file);
    let mut valid_records = Vec::new();
    let valid_categories = vec!["A", "B", "C", "D"];

    for result in rdr.deserialize() {
        let record: Record = result?;
        
        if let Some(cleaned_value) = clean_numeric_field(record.value, 1000.0) {
            if validate_category(&record.category, &valid_categories) {
                valid_records.push(record);
            }
        }
    }

    let mut wtr = csv::Writer::from_path(output_path)?;
    for record in valid_records {
        wtr.serialize(record)?;
    }
    wtr.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_numeric_field() {
        assert_eq!(clean_numeric_field(42.5, 1000.0), Some(42.5));
        assert_eq!(clean_numeric_field(1500.0, 1000.0), None);
        assert_eq!(clean_numeric_field(f64::INFINITY, 1000.0), None);
    }

    #[test]
    fn test_validate_category() {
        let categories = vec!["A", "B", "C"];
        assert!(validate_category("A", &categories));
        assert!(!validate_category("X", &categories));
    }
}