use std::collections::HashSet;

pub fn clean_string_list(input: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    input
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .filter_map(|s| {
            let trimmed = s.trim().to_string();
            if seen.insert(trimmed.clone()) {
                Some(trimmed)
            } else {
                None
            }
        })
        .collect()
}use csv::{Reader, Writer};
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

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_writer(File::create(output_path)?);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = record.name.trim().to_string();
        record.category = record.category.to_lowercase();
        
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
    record.value.is_finite() && 
    !record.category.is_empty()
}

fn main() -> Result<(), Box<dyn Error>> {
    let input = "raw_data.csv";
    let output = "cleaned_data.csv";
    
    match clean_csv(input, output) {
        Ok(_) => println!("Data cleaning completed successfully"),
        Err(e) => eprintln!("Error during cleaning: {}", e),
    }
    
    Ok(())
}