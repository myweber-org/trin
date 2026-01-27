
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0 && !self.category.is_empty()
    }

    fn transform(&mut self, multiplier: f64) {
        self.value *= multiplier;
        self.category = self.category.to_uppercase();
    }
}

fn process_csv_file(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(input_path);
    if !path.exists() {
        return Err("Input file does not exist".into());
    }

    let mut reader = Reader::from_path(input_path)?;
    let mut records: Vec<Record> = Vec::new();

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        if record.is_valid() {
            record.transform(1.5);
            records.push(record);
        }
    }

    if records.is_empty() {
        return Err("No valid records found".into());
    }

    let mut writer = Writer::from_path(output_path)?;
    for record in records {
        writer.serialize(&record)?;
    }

    writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    if count == 0.0 {
        return (0.0, 0.0, 0.0);
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

fn filter_by_category(records: Vec<Record>, category_filter: &str) -> Vec<Record> {
    records.into_iter()
        .filter(|r| r.category == category_filter)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 10.0,
            category: "A".to_string(),
        };
        assert!(valid_record.is_valid());

        let invalid_record = Record {
            id: 2,
            name: "".to_string(),
            value: -5.0,
            category: "B".to_string(),
        };
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = Record {
            id: 1,
            name: "test".to_string(),
            value: 10.0,
            category: "category".to_string(),
        };
        
        record.transform(2.0);
        assert_eq!(record.value, 20.0);
        assert_eq!(record.category, "CATEGORY");
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            Record { id: 1, name: "A".to_string(), value: 10.0, category: "X".to_string() },
            Record { id: 2, name: "B".to_string(), value: 20.0, category: "X".to_string() },
            Record { id: 3, name: "C".to_string(), value: 30.0, category: "Y".to_string() },
        ];
        
        let (sum, mean, std_dev) = calculate_statistics(&records);
        assert_eq!(sum, 60.0);
        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            records.push(Record::new(id, name, value, category));
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<&Record> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let total: f64 = records.iter().map(|record| record.value).sum();
    total / records.len() as f64
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record::new(1, "ItemA".to_string(), 10.5, "Alpha".to_string()),
            Record::new(2, "ItemB".to_string(), 20.3, "Beta".to_string()),
            Record::new(3, "ItemC".to_string(), 15.7, "Alpha".to_string()),
        ];

        let filtered = filter_by_category(&records, "Alpha");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record::new(1, "ItemA".to_string(), 10.0, "Test".to_string()),
            Record::new(2, "ItemB".to_string(), 20.0, "Test".to_string()),
            Record::new(3, "ItemC".to_string(), 30.0, "Test".to_string()),
        ];

        let avg = calculate_average(&records);
        assert_eq!(avg, 20.0);
    }

    #[test]
    fn test_find_max_value() {
        let records = vec![
            Record::new(1, "ItemA".to_string(), 10.5, "Test".to_string()),
            Record::new(2, "ItemB".to_string(), 25.3, "Test".to_string()),
            Record::new(3, "ItemC".to_string(), 15.7, "Test".to_string()),
        ];

        let max_record = find_max_value(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 25.3);
    }
}