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

    pub fn is_duplicate(&mut self, item: &str) -> bool {
        let normalized = self.normalize_text(item);
        if self.dedupe_set.contains(&normalized) {
            true
        } else {
            self.dedupe_set.insert(normalized);
            false
        }
    }

    pub fn clean_dataset(&mut self, data: Vec<String>) -> Vec<String> {
        let mut cleaned = Vec::new();
        
        for item in data {
            if !self.is_duplicate(&item) {
                cleaned.push(item);
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
        let data = vec![
            "Apple".to_string(),
            "apple".to_string(),
            "APPLE".to_string(),
            "Banana".to_string(),
        ];
        
        let cleaned = cleaner.clean_dataset(data);
        assert_eq!(cleaned.len(), 2);
        assert_eq!(cleaner.get_unique_count(), 2);
    }

    #[test]
    fn test_normalization() {
        let cleaner = DataCleaner::new();
        let result = cleaner.normalize_text("  HELLO World  ");
        assert_eq!(result, "hello world");
    }
}use csv::{ReaderBuilder, WriterBuilder};
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

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(Path::new(output_path))?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    let mut valid_count = 0;
    let mut invalid_count = 0;

    for result in rdr.deserialize() {
        let record: Record = match result {
            Ok(rec) => rec,
            Err(e) => {
                eprintln!("Failed to parse record: {}", e);
                invalid_count += 1;
                continue;
            }
        };

        if record.value.is_finite() && !record.name.is_empty() {
            wtr.serialize(&record)?;
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }

    wtr.flush()?;
    println!("Processing complete. Valid records: {}, Invalid records: {}", valid_count, invalid_count);
    Ok(())
}

fn main() {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";

    if let Err(e) = clean_csv_data(input_file, output_file) {
        eprintln!("Error during data cleaning: {}", e);
        std::process::exit(1);
    }
}use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: Option<u8>,
    score: Option<f64>,
}

fn clean_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(Path::new(input_path))?;
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(Path::new(output_path))?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        
        if record.name.is_empty() {
            record.name = "Unknown".to_string();
        }
        
        if record.age.is_none() || record.age.unwrap() > 120 {
            record.age = Some(0);
        }
        
        if record.score.is_none() || record.score.unwrap() < 0.0 {
            record.score = Some(0.0);
        }
        
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "input_data.csv";
    let output = "cleaned_data.csv";
    
    match clean_csv_data(input, output) {
        Ok(_) => println!("Data cleaning completed successfully"),
        Err(e) => eprintln!("Error during data cleaning: {}", e),
    }
    
    Ok(())
}