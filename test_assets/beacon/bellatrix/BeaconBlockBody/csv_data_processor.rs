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
struct AggregatedData {
    category: String,
    total_value: f64,
    average_value: f64,
    record_count: usize,
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

fn filter_active_records(records: &[Record]) -> Vec<&Record> {
    records.iter().filter(|r| r.active).collect()
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

fn write_aggregated_data<P: AsRef<Path>>(
    data: &[AggregatedData],
    path: P,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut writer = Writer::from_writer(file);

    for item in data {
        writer.serialize(item)?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let records = read_csv_file(input_path)?;
    let active_records = filter_active_records(&records);
    let aggregated_data = aggregate_by_category(&active_records);
    write_aggregated_data(&aggregated_data, output_path)?;

    println!("Processed {} records", records.len());
    println!("Found {} active records", active_records.len());
    println!("Generated {} aggregated categories", aggregated_data.len());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";

    match process_csv_data(input_file, output_file) {
        Ok(_) => println!("CSV processing completed successfully"),
        Err(e) => eprintln!("Error processing CSV: {}", e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_active_records() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Category1".to_string(),
                value: 10.5,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Category2".to_string(),
                value: 20.0,
                active: false,
            },
        ];

        let active = filter_active_records(&records);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, 1);
    }

    #[test]
    fn test_aggregate_by_category() {
        let records = vec![
            Record {
                id: 1,
                name: "Item A".to_string(),
                category: "Category1".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Item B".to_string(),
                category: "Category1".to_string(),
                value: 20.0,
                active: true,
            },
        ];

        let aggregated = aggregate_by_category(&records);
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].total_value, 30.0);
        assert_eq!(aggregated[0].average_value, 15.0);
        assert_eq!(aggregated[0].record_count, 2);
    }

    #[test]
    fn test_csv_roundtrip() -> Result<(), Box<dyn Error>> {
        let temp_input = NamedTempFile::new()?;
        let temp_output = NamedTempFile::new()?;

        let test_data = "id,name,category,value,active\n1,Test Item,TestCat,15.5,true\n";
        std::fs::write(temp_input.path(), test_data)?;

        process_csv_data(
            temp_input.path().to_str().unwrap(),
            temp_output.path().to_str().unwrap(),
        )?;

        assert!(temp_output.path().exists());
        Ok(())
    }
}
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

pub fn process_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(input_path)?;
    let mut reader = Reader::from_reader(input_file);
    
    let output_file = File::create(output_path)?;
    let mut writer = Writer::from_writer(output_file);

    let mut total_value = 0.0;
    let mut active_count = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        
        if record.active && record.value > 50.0 {
            total_value += record.value;
            active_count += 1;
            
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    
    if active_count > 0 {
        let average_value = total_value / active_count as f64;
        println!("Processed {} active records with average value: {:.2}", active_count, average_value);
    } else {
        println!("No records matched the filter criteria");
    }

    Ok(())
}

pub fn filter_by_category(records: Vec<Record>, category: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category && r.active)
        .collect()
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let average = sum / count;
    
    let max = records.iter()
        .map(|r| r.value)
        .fold(f64::NEG_INFINITY, f64::max);
    
    let min = records.iter()
        .map(|r| r.value)
        .fold(f64::INFINITY, f64::min);

    (average, min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record { id: 1, name: "Item A".to_string(), category: "Electronics".to_string(), value: 100.0, active: true },
            Record { id: 2, name: "Item B".to_string(), category: "Books".to_string(), value: 25.0, active: true },
            Record { id: 3, name: "Item C".to_string(), category: "Electronics".to_string(), value: 75.0, active: false },
        ];

        let filtered = filter_by_category(records, "Electronics");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Item A");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), category: "Test".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Test2".to_string(), category: "Test".to_string(), value: 20.0, active: true },
            Record { id: 3, name: "Test3".to_string(), category: "Test".to_string(), value: 30.0, active: true },
        ];

        let (avg, min, max) = calculate_statistics(&records);
        assert_eq!(avg, 20.0);
        assert_eq!(min, 10.0);
        assert_eq!(max, 30.0);
    }
}