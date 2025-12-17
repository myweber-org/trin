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

fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
}

fn calculate_category_average(records: &[Record], category: &str) -> Option<f64> {
    let filtered: Vec<&Record> = records
        .iter()
        .filter(|r| r.category == category && r.active)
        .collect();

    if filtered.is_empty() {
        return None;
    }

    let sum: f64 = filtered.iter().map(|r| r.value).sum();
    Some(sum / filtered.len() as f64)
}

fn write_filtered_records(records: &[&Record], output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    for record in records {
        writer.serialize(record)?;
    }

    writer.flush()?;
    Ok(())
}

fn process_data_pipeline(input_file: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_file)?;
    
    println!("Total records loaded: {}", records.len());
    
    let active_records = filter_active_records(&records);
    println!("Active records: {}", active_records.len());
    
    if let Some(avg) = calculate_category_average(&records, "premium") {
        println!("Average value for premium category: {:.2}", avg);
    }
    
    write_filtered_records(&active_records, output_file)?;
    println!("Filtered records written to: {}", output_file);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/active_records.csv";
    
    process_data_pipeline(input_file, output_file)
}