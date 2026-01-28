use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

pub fn process_csv_file(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        
        if line_number == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 3 {
            continue;
        }

        let id = match parts[0].parse::<u32>() {
            Ok(id) => id,
            Err(_) => continue,
        };

        let value = match parts[1].parse::<f64>() {
            Ok(value) => value,
            Err(_) => continue,
        };

        let category = parts[2].to_string();

        if value < 0.0 {
            continue;
        }

        records.push(DataRecord {
            id,
            value,
            category,
        });
    }

    Ok(records)
}

pub fn calculate_average(records: &[DataRecord]) -> Option<f64> {
    if records.is_empty() {
        return None;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    Some(sum / records.len() as f64)
}

pub fn filter_by_category(records: Vec<DataRecord>, category: &str) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|r| r.category == category)
        .collect()
}