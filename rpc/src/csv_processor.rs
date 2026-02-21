use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

fn load_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    
    let mut rdr = csv::Reader::from_reader(reader);
    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter()
        .filter(|r| r.active && r.value > 0.0)
        .collect()
}

fn save_filtered_csv<P: AsRef<Path>>(records: &[&Record], path: P) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut wtr = csv::Writer::from_writer(writer);
    
    for record in records {
        wtr.serialize(record)?;
    }
    
    wtr.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[&Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let avg = sum / count;
    let max = records.iter().map(|r| r.value).fold(f64::NEG_INFINITY, f64::max);
    
    (sum, avg, max)
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_path)?;
    let filtered = filter_active_records(&records);
    
    if filtered.is_empty() {
        println!("No active records found with positive values");
        return Ok(());
    }
    
    let (total, average, maximum) = calculate_statistics(&filtered);
    println!("Processed {} records", filtered.len());
    println!("Total value: {:.2}", total);
    println!("Average value: {:.2}", average);
    println!("Maximum value: {:.2}", maximum);
    
    save_filtered_csv(&filtered, output_path)?;
    println!("Filtered data saved to {}", output_path);
    
    Ok(())
}

fn main() {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    
    if let Err(e) = process_csv(input_file, output_file) {
        eprintln!("Error processing CSV: {}", e);
        std::process::exit(1);
    }
}