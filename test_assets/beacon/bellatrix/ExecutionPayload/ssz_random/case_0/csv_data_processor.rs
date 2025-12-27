
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

#[derive(Debug)]
struct AggregatedData {
    category: String,
    total_value: f64,
    average_value: f64,
    record_count: usize,
}

fn load_csv_data<P: AsRef<Path>>(file_path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let mut reader = Reader::from_path(file_path)?;
    let mut records = Vec::new();
    
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter()
        .filter(|record| record.active)
        .collect()
}

fn aggregate_by_category(records: &[Record]) -> Vec<AggregatedData> {
    use std::collections::HashMap;
    
    let mut category_map: HashMap<String, (f64, usize)> = HashMap::new();
    
    for record in records {
        let entry = category_map.entry(record.category.clone()).or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }
    
    category_map.into_iter()
        .map(|(category, (total, count))| AggregatedData {
            category,
            total_value: total,
            average_value: total / count as f64,
            record_count: count,
        })
        .collect()
}

fn write_aggregated_data<P: AsRef<Path>>(
    aggregated_data: &[AggregatedData],
    output_path: P
) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(output_path)?;
    
    for data in aggregated_data {
        writer.serialize(data)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = load_csv_data(input_path)?;
    
    println!("Loaded {} total records", records.len());
    
    let active_records = filter_active_records(&records);
    println!("Found {} active records", active_records.len());
    
    let aggregated_data = aggregate_by_category(&active_records);
    
    for data in &aggregated_data {
        println!(
            "Category: {}, Total: {:.2}, Average: {:.2}, Count: {}",
            data.category, data.total_value, data.average_value, data.record_count
        );
    }
    
    write_aggregated_data(&aggregated_data, output_path)?;
    println!("Results written to {}", output_path);
    
    Ok(())
}

pub fn run_data_processing() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output_aggregated.csv";
    
    process_csv_data(input_file, output_file)
}