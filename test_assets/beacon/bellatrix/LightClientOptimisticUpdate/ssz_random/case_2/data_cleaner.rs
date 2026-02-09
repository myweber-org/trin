
use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
            && self.value >= 0.0
            && !self.category.trim().is_empty()
    }

    fn clean(&mut self) {
        self.name = self.name.trim().to_string();
        self.category = self.category.trim().to_string();
        if self.value < 0.0 {
            self.value = 0.0;
        }
    }
}

pub fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(Path::new(output_path))?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let mut valid_count = 0;
    let mut cleaned_count = 0;

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        if !record.is_valid() {
            record.clean();
            cleaned_count += 1;
        }
        
        if record.is_valid() {
            writer.serialize(&record)?;
            valid_count += 1;
        }
    }

    println!("Processed {} records", valid_count + cleaned_count);
    println!("Valid records: {}", valid_count);
    println!("Cleaned records: {}", cleaned_count);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        assert!(valid_record.is_valid());

        let invalid_record = Record {
            id: 2,
            name: "   ".to_string(),
            value: -5.0,
            category: "".to_string(),
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_clean_csv_data() -> Result<(), Box<dyn Error>> {
        let input_data = "id,name,value,category\n1,Test,10.5,A\n2,  ,-5.0,  \n";
        let input_file = NamedTempFile::new()?;
        std::fs::write(input_file.path(), input_data)?;

        let output_file = NamedTempFile::new()?;
        
        clean_csv_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        )?;

        let output_content = std::fs::read_to_string(output_file.path())?;
        assert!(output_content.contains("Test,10.5,A"));
        assert!(!output_content.contains("-5.0"));

        Ok(())
    }
}use std::collections::HashSet;

pub struct DataCleaner {
    deduplication_cache: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            deduplication_cache: HashSet::new(),
        }
    }

    pub fn normalize_text(&self, input: &str) -> String {
        input
            .trim()
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect()
    }

    pub fn is_duplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.deduplication_cache.contains(&normalized) {
            true
        } else {
            self.deduplication_cache.insert(normalized);
            false
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<String>) -> Vec<String> {
        let mut cleaned = Vec::new();
        for item in data {
            if !self.is_duplicate(&item) {
                cleaned.push(self.normalize_text(&item));
            }
        }
        cleaned
    }

    pub fn reset_cache(&mut self) {
        self.deduplication_cache.clear();
    }

    pub fn get_unique_count(&self) -> usize {
        self.deduplication_cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        assert_eq!(cleaner.normalize_text("  Hello WORLD!  "), "hello world");
        assert_eq!(cleaner.normalize_text("Data@2024"), "data2024");
    }

    #[test]
    fn test_deduplication() {
        let mut cleaner = DataCleaner::new();
        assert!(!cleaner.is_duplicate("test"));
        assert!(cleaner.is_duplicate("TEST"));
        assert!(!cleaner.is_duplicate("another"));
    }

    #[test]
    fn test_clean_dataset() {
        let mut cleaner = DataCleaner::new();
        let data = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "Banana".to_string(),
            "  banana  ".to_string(),
        ];
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
    }
}