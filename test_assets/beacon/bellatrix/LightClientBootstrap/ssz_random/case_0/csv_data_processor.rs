
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
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
struct ProcessedData {
    total_records: usize,
    average_value: f64,
    categories: Vec<String>,
    active_count: usize,
}

fn read_csv_file<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_records(records: &[Record], category_filter: Option<&str>, active_only: bool) -> Vec<Record> {
    records
        .iter()
        .filter(|record| {
            let category_match = category_filter
                .map(|filter| record.category == filter)
                .unwrap_or(true);
            
            let active_match = !active_only || record.active;
            
            category_match && active_match
        })
        .cloned()
        .collect()
}

fn process_data(records: &[Record]) -> ProcessedData {
    let total_records = records.len();
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let average_value = if total_records > 0 {
        sum / total_records as f64
    } else {
        0.0
    };
    
    let mut categories: Vec<String> = records
        .iter()
        .map(|r| r.category.clone())
        .collect();
    categories.sort();
    categories.dedup();
    
    let active_count = records.iter().filter(|r| r.active).count();

    ProcessedData {
        total_records,
        average_value,
        categories,
        active_count,
    }
}

fn write_processed_data<P: AsRef<Path>>(
    path: P,
    data: &ProcessedData,
    filtered_records: &[Record]
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = Writer::from_writer(file);

    writer.write_record(&[
        "total_records",
        "average_value",
        "category_count",
        "active_count",
    ])?;

    writer.write_record(&[
        data.total_records.to_string(),
        format!("{:.2}", data.average_value),
        data.categories.len().to_string(),
        data.active_count.to_string(),
    ])?;

    writer.write_record(&[])?;
    writer.write_record(&["Filtered Records"])?;
    writer.write_record(&["id", "name", "category", "value", "active"])?;

    for record in filtered_records {
        writer.serialize(record)?;
    }

    writer.flush()?;
    Ok(())
}

pub fn process_csv_file(
    input_path: &str,
    output_path: &str,
    category_filter: Option<&str>,
    active_only: bool
) -> Result<ProcessedData, Box<dyn Error>> {
    let records = read_csv_file(input_path)?;
    let filtered_records = filter_records(&records, category_filter, active_only);
    let processed_data = process_data(&filtered_records);
    
    write_processed_data(output_path, &processed_data, &filtered_records)?;
    
    Ok(processed_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_records() {
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
                category: "Books".to_string(),
                value: 50.0,
                active: false,
            },
        ];

        let filtered = filter_records(&records, Some("Electronics"), true);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_process_data() {
        let records = vec![
            Record {
                id: 1,
                name: "Test".to_string(),
                category: "A".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Test2".to_string(),
                category: "B".to_string(),
                value: 20.0,
                active: true,
            },
        ];

        let data = process_data(&records);
        assert_eq!(data.total_records, 2);
        assert_eq!(data.average_value, 15.0);
        assert_eq!(data.categories.len(), 2);
        assert_eq!(data.active_count, 2);
    }

    #[test]
    fn test_full_processing() -> Result<(), Box<dyn Error>> {
        let input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;
        
        let input_path = input_file.path().to_str().unwrap();
        let output_path = output_file.path().to_str().unwrap();

        let mut writer = Writer::from_writer(&input_file);
        writer.write_record(&["id", "name", "category", "value", "active"])?;
        writer.write_record(&["1", "Test", "A", "10.5", "true"])?;
        writer.write_record(&["2", "Test2", "B", "20.5", "false"])?;
        writer.flush()?;

        let result = process_csv_file(input_path, output_path, Some("A"), true)?;
        assert_eq!(result.total_records, 1);
        
        Ok(())
    }
}