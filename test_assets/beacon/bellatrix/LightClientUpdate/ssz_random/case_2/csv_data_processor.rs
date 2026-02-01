use csv::{ReaderBuilder, WriterBuilder};
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

fn process_csv(input_path: &str, output_path: &str, min_value: f64) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = WriterBuilder::new().from_writer(output_file);
    
    writer.write_record(&["id", "name", "category", "value", "active", "processed_value"])?;
    
    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.value >= min_value && record.active {
            let processed_value = record.value * 1.15;
            writer.serialize((
                record.id,
                record.name,
                record.category,
                record.value,
                record.active,
                processed_value,
            ))?;
        }
    }
    
    writer.flush()?;
    Ok(())
}

fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64, usize)> {
    use std::collections::HashMap;
    
    let mut aggregates: HashMap<String, (f64, usize)> = HashMap::new();
    
    for record in records {
        let entry = aggregates.entry(record.category.clone()).or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }
    
    aggregates
        .into_iter()
        .map(|(category, (total, count))| (category, total, count))
        .collect()
}

fn validate_record(record: &Record) -> Result<(), String> {
    if record.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    
    if record.value < 0.0 {
        return Err("Value cannot be negative".to_string());
    }
    
    if record.category.len() > 50 {
        return Err("Category name too long".to_string());
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";
    
    match process_csv(input_file, output_file, 100.0) {
        Ok(_) => println!("Processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_record_valid() {
        let record = Record {
            id: 1,
            name: "Test Item".to_string(),
            category: "Electronics".to_string(),
            value: 150.0,
            active: true,
        };
        
        assert!(validate_record(&record).is_ok());
    }
    
    #[test]
    fn test_validate_record_invalid_name() {
        let record = Record {
            id: 2,
            name: "   ".to_string(),
            category: "Books".to_string(),
            value: 50.0,
            active: true,
        };
        
        assert!(validate_record(&record).is_err());
    }
    
    #[test]
    fn test_aggregate_by_category() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Electronics".to_string(),
                value: 100.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Electronics".to_string(),
                value: 200.0,
                active: true,
            },
            Record {
                id: 3,
                name: "Item C".to_string(),
                category: "Books".to_string(),
                value: 50.0,
                active: true,
            },
        ];
        
        let aggregates = aggregate_by_category(&records);
        assert_eq!(aggregates.len(), 2);
        
        let electronics_agg = aggregates.iter().find(|(cat, _, _)| cat == "Electronics").unwrap();
        assert_eq!(electronics_agg.1, 300.0);
        assert_eq!(electronics_agg.2, 2);
    }
}