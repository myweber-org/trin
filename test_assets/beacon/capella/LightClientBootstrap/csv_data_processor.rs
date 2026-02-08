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

fn filter_records(records: &[Record], category_filter: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|r| r.category == category_filter && r.active)
        .collect()
}

fn calculate_average(records: &[&Record]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }
    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

fn process_csv_file(input_path: &str, output_path: &str, target_category: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut rdr = Reader::from_reader(file);
    
    let mut all_records = Vec::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        all_records.push(record);
    }
    
    let filtered = filter_records(&all_records, target_category);
    let avg_value = calculate_average(&filtered);
    
    let output_file = File::create(output_path)?;
    let mut wtr = Writer::from_writer(output_file);
    
    for record in filtered {
        wtr.serialize(record)?;
    }
    
    wtr.flush()?;
    
    if let Some(avg) = avg_value {
        println!("Processed {} records with category '{}'", filtered.len(), target_category);
        println!("Average value: {:.2}", avg);
    } else {
        println!("No records found for category '{}'", target_category);
    }
    
    Ok(())
}

fn generate_sample_data() -> Result<(), Box<dyn Error>> {
    let records = vec![
        Record { id: 1, name: "Item A".to_string(), category: "Electronics".to_string(), value: 99.99, active: true },
        Record { id: 2, name: "Item B".to_string(), category: "Books".to_string(), value: 24.50, active: true },
        Record { id: 3, name: "Item C".to_string(), category: "Electronics".to_string(), value: 149.99, active: false },
        Record { id: 4, name: "Item D".to_string(), category: "Electronics".to_string(), value: 79.99, active: true },
        Record { id: 5, name: "Item E".to_string(), category: "Clothing".to_string(), value: 45.00, active: true },
    ];
    
    let file = File::create("sample_data.csv")?;
    let mut wtr = Writer::from_writer(file);
    
    for record in records {
        wtr.serialize(record)?;
    }
    
    wtr.flush()?;
    println!("Sample data generated: sample_data.csv");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    generate_sample_data()?;
    
    match process_csv_file("sample_data.csv", "filtered_electronics.csv", "Electronics") {
        Ok(_) => println!("Processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }
    
    Ok(())
}