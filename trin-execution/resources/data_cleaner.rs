use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.category = record.category.to_uppercase();
        
        if record.value < 0.0 {
            record.value = 0.0;
        }
        
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && 
    record.value >= 0.0 && 
    !record.category.is_empty()
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "data/raw.csv";
    let output = "data/cleaned.csv";
    
    match clean_csv_data(input, output) {
        Ok(_) => println!("Data cleaning completed successfully"),
        Err(e) => eprintln!("Error during data cleaning: {}", e),
    }
    
    Ok(())
}use std::collections::HashSet;

pub struct DataCleaner;

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner
    }

    pub fn deduplicate_strings(&self, strings: &[String]) -> Vec<String> {
        let mut seen = HashSet::new();
        strings
            .iter()
            .filter(|s| seen.insert(s.clone()))
            .cloned()
            .collect()
    }

    pub fn normalize_strings(&self, strings: &[String]) -> Vec<String> {
        strings
            .iter()
            .map(|s| s.trim().to_lowercase())
            .collect()
    }

    pub fn clean_data(&self, raw_data: &[String]) -> Vec<String> {
        let normalized = self.normalize_strings(raw_data);
        self.deduplicate_strings(&normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deduplication() {
        let cleaner = DataCleaner::new();
        let input = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apple".to_string(),
            "Banana".to_string(),
        ];
        let result = cleaner.deduplicate_strings(&input);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        let input = vec![
            "  Apple  ".to_string(),
            "BANANA".to_string(),
            "  Cherry  ".to_string(),
        ];
        let result = cleaner.normalize_strings(&input);
        assert_eq!(result[0], "apple");
        assert_eq!(result[1], "banana");
        assert_eq!(result[2], "cherry");
    }

    #[test]
    fn test_full_clean() {
        let cleaner = DataCleaner::new();
        let input = vec![
            "  Apple  ".to_string(),
            "apple".to_string(),
            "BANANA".to_string(),
            "banana  ".to_string(),
            "  Cherry  ".to_string(),
        ];
        let result = cleaner.clean_data(&input);
        assert_eq!(result, vec!["apple", "banana", "cherry"]);
    }
}