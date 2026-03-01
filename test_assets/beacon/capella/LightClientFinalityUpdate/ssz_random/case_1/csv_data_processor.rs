use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
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
    let mut reader = Reader::from_path(path)?;
    let mut records = Vec::new();
    
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter()
        .filter(|r| r.active)
        .collect()
}

fn calculate_category_totals(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;
    
    let mut totals: HashMap<String, f64> = HashMap::new();
    
    for record in records {
        *totals.entry(record.category.clone()).or_insert(0.0) += record.value;
    }
    
    totals.into_iter().collect()
}

fn save_results<P: AsRef<Path>>(path: P, totals: &[(String, f64)]) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(path)?;
    
    writer.write_record(&["Category", "Total"])?;
    
    for (category, total) in totals {
        writer.write_record(&[category, &total.to_string()])?;
    }
    
    writer.flush()?;
    Ok(())
}

fn process_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv(input_path)?;
    let active_records = filter_active_records(&records);
    let category_totals = calculate_category_totals(&active_records);
    
    save_results(output_path, &category_totals)?;
    
    println!("Processed {} records", records.len());
    println!("Found {} active records", active_records.len());
    println!("Generated totals for {} categories", category_totals.len());
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    
    process_data(input_file, output_file)
}