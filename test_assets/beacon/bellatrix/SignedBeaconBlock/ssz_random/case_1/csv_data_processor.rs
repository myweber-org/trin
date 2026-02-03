
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
    category: String,
    total_value: f64,
    average_value: f64,
    record_count: usize,
    active_count: usize,
}

fn read_csv_data<P: AsRef<Path>>(file_path: P) -> Result<Vec<Record>, Box<dyn Error>> {
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

fn aggregate_by_category(records: &[Record]) -> Vec<ProcessedData> {
    use std::collections::HashMap;

    let mut category_map: HashMap<String, (f64, usize, usize)> = HashMap::new();

    for record in records {
        let entry = category_map.entry(record.category.clone()).or_insert((0.0, 0, 0));
        entry.0 += record.value;
        entry.1 += 1;
        if record.active {
            entry.2 += 1;
        }
    }

    category_map
        .into_iter()
        .map(|(category, (total_value, count, active_count))| ProcessedData {
            category,
            total_value,
            average_value: total_value / count as f64,
            record_count: count,
            active_count,
        })
        .collect()
}

fn write_processed_data<P: AsRef<Path>>(
    processed_data: &[ProcessedData],
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    for data in processed_data {
        writer.serialize(data)?;
    }

    writer.flush()?;
    Ok(())
}

fn process_csv_data(input_file: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    let records = read_csv_data(input_file)?;
    let active_records = filter_active_records(&records);
    let aggregated_data = aggregate_by_category(&records);

    println!("Total records: {}", records.len());
    println!("Active records: {}", active_records.len());
    println!("Categories found: {}", aggregated_data.len());

    for data in &aggregated_data {
        println!(
            "Category: {}, Total: {:.2}, Average: {:.2}, Records: {}, Active: {}",
            data.category,
            data.total_value,
            data.average_value,
            data.record_count,
            data.active_count
        );
    }

    write_processed_data(&aggregated_data, output_file)?;
    println!("Processed data written to: {}", output_file);

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
                name: "Test1".to_string(),
                category: "A".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Test2".to_string(),
                category: "B".to_string(),
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
                name: "Test1".to_string(),
                category: "A".to_string(),
                value: 10.0,
                active: true,
            },
            Record {
                id: 2,
                name: "Test2".to_string(),
                category: "A".to_string(),
                value: 20.0,
                active: false,
            },
        ];

        let aggregated = aggregate_by_category(&records);
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].category, "A");
        assert_eq!(aggregated[0].total_value, 30.0);
        assert_eq!(aggregated[0].record_count, 2);
        assert_eq!(aggregated[0].active_count, 1);
    }

    #[test]
    fn test_write_processed_data() -> Result<(), Box<dyn Error>> {
        let processed_data = vec![ProcessedData {
            category: "Test".to_string(),
            total_value: 100.0,
            average_value: 50.0,
            record_count: 2,
            active_count: 1,
        }];

        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();

        write_processed_data(&processed_data, path)?;

        let file_content = std::fs::read_to_string(path)?;
        assert!(file_content.contains("Test"));
        assert!(file_content.contains("100"));
        assert!(file_content.contains("50"));

        Ok(())
    }
}