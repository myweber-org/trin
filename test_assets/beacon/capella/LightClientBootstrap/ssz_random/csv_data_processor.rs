
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

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if record.value < 0.0 {
        return Err("Value must be non-negative".to_string());
    }
    if !["A", "B", "C"].contains(&record.category.as_str()) {
        return Err("Category must be A, B, or C".to_string());
    }
    Ok(())
}

fn transform_record(record: &mut Record) {
    record.name = record.name.to_uppercase();
    record.value = (record.value * 100.0).round() / 100.0;
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        match validate_record(&record) {
            Ok(_) => {
                transform_record(&mut record);
                writer.serialize(&record)?;
            }
            Err(e) => eprintln!("Skipping invalid record: {}", e),
        }
    }

    writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = if count > 0.0 { sum / count } else { 0.0 };
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/processed.csv";
    
    process_csv(input_file, output_file)?;
    
    let file = File::open(output_file)?;
    let mut reader = Reader::from_reader(file);
    let records: Vec<Record> = reader.deserialize().collect::<Result<_, _>>()?;
    
    let (total, average, deviation) = calculate_statistics(&records);
    println!("Statistics - Total: {:.2}, Average: {:.2}, Std Dev: {:.2}", 
             total, average, deviation);
    
    Ok(())
}