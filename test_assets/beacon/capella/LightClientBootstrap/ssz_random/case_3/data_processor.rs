use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err("File does not exist".into());
    }

    let mut reader = Reader::from_path(path)?;
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value < 0.0 {
            return Err(format!("Invalid value in record ID {}", record.id).into());
        }
        
        records.push(record);
    }

    if records.is_empty() {
        return Err("No valid records found".into());
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (mean, variance, std_dev)
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category)
        .collect()
}