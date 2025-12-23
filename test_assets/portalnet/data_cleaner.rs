use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

fn clean_csv_data(input_path: &Path, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input_file);

    let output_file = File::create(output_path)?;
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_writer(output_file);

    for result in rdr.deserialize() {
        let record: Record = result?;
        
        let cleaned_record = Record {
            id: record.id,
            name: record.name.trim().to_string(),
            age: if record.age > 120 { 120 } else { record.age },
            active: record.active,
        };

        wtr.serialize(cleaned_record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.age > 0 && record.age <= 120
}

pub fn process_dataset(input: &str, output: &str) -> Result<(), Box<dyn Error>> {
    let input_path = Path::new(input);
    let output_path = Path::new(output);
    
    if !input_path.exists() {
        return Err("Input file does not exist".into());
    }

    clean_csv_data(input_path, output_path)?;
    
    let output_file = File::open(output_path)?;
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(output_file);

    let mut valid_count = 0;
    let mut total_count = 0;

    for result in rdr.deserialize() {
        let record: Record = result?;
        total_count += 1;
        
        if validate_record(&record) {
            valid_count += 1;
        }
    }

    println!("Processed {} records, {} valid", total_count, valid_count);
    Ok(())
}
use std::collections::HashSet;

pub fn clean_and_sort_data(input: Vec<String>) -> Vec<String> {
    let mut unique_items: HashSet<String> = input.into_iter().collect();
    let mut sorted_items: Vec<String> = unique_items.into_iter().collect();
    sorted_items.sort();
    sorted_items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_and_sort_data() {
        let input = vec![
            "banana".to_string(),
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
            "apple".to_string(),
        ];
        
        let result = clean_and_sort_data(input);
        let expected = vec![
            "apple".to_string(),
            "banana".to_string(),
            "cherry".to_string(),
        ];
        
        assert_eq!(result, expected);
    }
}