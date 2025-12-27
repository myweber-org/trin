use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
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
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }
}

pub fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
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
        let record: Record = result?;
        if record.is_valid() {
            valid_records.push(record);
        } else {
            invalid_count += 1;
        }
    }

    if valid_records.is_empty() {
        return Err("No valid records found".into());
    }

    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_path(output_path)?;

    for record in valid_records {
        writer.serialize(record)?;
    }

    writer.flush()?;
    println!("Processed {} records, filtered {} invalid entries", valid_records.len(), invalid_count);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 42.5,
            category: "A".to_string(),
        };
        assert!(valid_record.is_valid());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -1.0,
            category: "B".to_string(),
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_csv_processing() {
        let test_input = "test_input.csv";
        let test_output = "test_output.csv";
        
        let csv_data = "id,name,value,category\n1,Alice,100.5,X\n2,Bob,-50.0,Y\n3,,75.0,Z\n";
        fs::write(test_input, csv_data).unwrap();

        let result = process_csv(test_input, test_output);
        assert!(result.is_ok());

        let output_content = fs::read_to_string(test_output).unwrap();
        assert!(output_content.contains("Alice"));
        assert!(!output_content.contains("Bob"));
        assert!(!output_content.contains(",,"));

        fs::remove_file(test_input).unwrap();
        fs::remove_file(test_output).unwrap();
    }
}