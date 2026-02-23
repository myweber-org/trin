use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn filter_records_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category && record.active)
        .collect()
}

fn calculate_average_value(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }
    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

fn process_csv_file(input_path: &str, output_path: &str, target_category: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut records: Vec<Record> = Vec::new();
    
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    let filtered = filter_records_by_category(&records, target_category);
    let avg_value = calculate_average_value(&filtered);
    
    let mut writer = Writer::from_writer(File::create(output_path)?);
    
    for record in filtered {
        writer.serialize(record)?;
    }
    
    if let Some(avg) = avg_value {
        println!("Average value for category '{}': {:.2}", target_category, avg);
    } else {
        println!("No active records found for category '{}'", target_category);
    }
    
    println!("Processed {} records, filtered {} records", records.len(), filtered.len());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    let target_category = "electronics";
    
    process_csv_file(input_file, output_file, target_category)?;
    
    Ok(())
}