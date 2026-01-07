use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl DataRecord {
    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty() &&
        self.value >= 0.0 &&
        !self.category.trim().is_empty()
    }
}

pub fn process_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(input_path);
    if !path.exists() {
        return Err("Input file does not exist".into());
    }

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(input_path)?;

    let mut valid_records = Vec::new();
    let mut invalid_count = 0;

    for result in reader.deserialize() {
        let record: DataRecord = result?;
        
        if record.is_valid() {
            valid_records.push(record);
        } else {
            invalid_count += 1;
        }
    }

    if valid_records.is_empty() {
        return Err("No valid records found in input file".into());
    }

    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_path(output_path)?;

    for record in valid_records {
        writer.serialize(&record)?;
    }

    writer.flush()?;

    println!("Processing complete:");
    println!("  Valid records: {}", valid_records.len());
    println!("  Invalid records: {}", invalid_count);
    println!("  Output written to: {}", output_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_valid_record() {
        let record = DataRecord {
            id: 1,
            name: "Test".to_string(),
            value: 10.5,
            category: "A".to_string(),
        };
        assert!(record.is_valid());
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "".to_string(),
        };
        assert!(!record.is_valid());
    }
}