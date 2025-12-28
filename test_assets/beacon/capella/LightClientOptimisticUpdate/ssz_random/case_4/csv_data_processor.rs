use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, category: &str, value: f64, active: bool) -> Self {
        DataRecord {
            id,
            category: category.to_string(),
            value,
            active,
        }
    }
}

fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: DataRecord = result?;
        records.push(record);
    }

    Ok(records)
}

fn filter_active_records(records: &[DataRecord]) -> Vec<DataRecord> {
    records
        .iter()
        .filter(|r| r.active)
        .cloned()
        .collect()
}

fn calculate_category_averages(records: &[DataRecord]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut category_sums: HashMap<String, (f64, usize)> = HashMap::new();

    for record in records {
        let entry = category_sums
            .entry(record.category.clone())
            .or_insert((0.0, 0));
        entry.0 += record.value;
        entry.1 += 1;
    }

    category_sums
        .into_iter()
        .map(|(category, (sum, count))| (category, sum / count as f64))
        .collect()
}

fn save_processed_data(
    file_path: &str,
    averages: &[(String, f64)],
) -> Result<(), Box<dyn Error>> {
    let file = File::create(file_path)?;
    let mut writer = Writer::from_writer(file);

    writer.write_record(&["Category", "AverageValue"])?;

    for (category, avg) in averages {
        writer.write_record(&[category, &avg.to_string()])?;
    }

    writer.flush()?;
    Ok(())
}

fn process_data_pipeline(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let all_records = load_csv_data(input_path)?;
    let active_records = filter_active_records(&all_records);
    let category_averages = calculate_category_averages(&active_records);
    save_processed_data(output_path, &category_averages)?;

    println!("Processed {} records", all_records.len());
    println!("Found {} active records", active_records.len());
    println!("Calculated averages for {} categories", category_averages.len());

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/output.csv";

    match process_data_pipeline(input_file, output_file) {
        Ok(_) => println!("Data processing completed successfully"),
        Err(e) => eprintln!("Error processing data: {}", e),
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
            DataRecord::new(1, "A", 10.0, true),
            DataRecord::new(2, "B", 20.0, false),
            DataRecord::new(3, "A", 30.0, true),
        ];

        let active = filter_active_records(&records);
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|r| r.active));
    }

    #[test]
    fn test_calculate_category_averages() {
        let records = vec![
            DataRecord::new(1, "A", 10.0, true),
            DataRecord::new(2, "A", 20.0, true),
            DataRecord::new(3, "B", 30.0, true),
        ];

        let averages = calculate_category_averages(&records);
        assert_eq!(averages.len(), 2);

        let a_avg = averages.iter().find(|(cat, _)| cat == "A").unwrap();
        assert_eq!(a_avg.1, 15.0);
    }
}