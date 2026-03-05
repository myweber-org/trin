
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

#[derive(Debug)]
struct AggregatedData {
    category: String,
    total_value: f64,
    average_value: f64,
    record_count: usize,
}

fn filter_records(records: &[Record], min_value: f64, active_only: bool) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.value >= min_value)
        .filter(|r| !active_only || r.active)
        .cloned()
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
    
    category_map
        .into_iter()
        .map(|(category, (total, count))| AggregatedData {
            category,
            total_value: total,
            average_value: total / count as f64,
            record_count: count,
        })
        .collect()
}

fn process_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut reader = Reader::from_reader(file);
    
    let mut records: Vec<Record> = Vec::new();
    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    let filtered_records = filter_records(&records, 50.0, true);
    
    let aggregated_data = aggregate_by_category(&filtered_records);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);
    
    for data in aggregated_data {
        writer.serialize(data)?;
    }
    
    writer.flush()?;
    Ok(())
}

fn validate_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_records() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Electronics".to_string(),
                value: 75.5,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Books".to_string(),
                value: 25.0,
                active: true,
            },
        ];
        
        let filtered = filter_records(&records, 50.0, true);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
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
        ];
        
        let aggregated = aggregate_by_category(&records);
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].total_value, 300.0);
        assert_eq!(aggregated[0].average_value, 150.0);
    }

    #[test]
    fn test_validate_record() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            category: "Test".to_string(),
            value: 10.0,
            active: true,
        };
        
        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            category: "Test".to_string(),
            value: -5.0,
            active: true,
        };
        
        assert!(validate_record(&valid_record));
        assert!(!validate_record(&invalid_record));
    }
}