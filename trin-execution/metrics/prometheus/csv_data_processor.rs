use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn load_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        if index == 0 {
            continue;
        }

        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() >= 4 {
            let id = parts[0].parse::<u32>()?;
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>()?;
            let category = parts[3].to_string();

            records.push(Record {
                id,
                name,
                value,
                category,
            });
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[Record], category: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}

pub fn calculate_average(records: &[Record]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let sum: f64 = records.iter().map(|r| r.value).sum();
    sum / records.len() as f64
}

pub fn find_max_value(records: &[Record]) -> Option<&Record> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

pub fn aggregate_by_category(records: &[Record]) -> Vec<(String, f64)> {
    use std::collections::HashMap;

    let mut category_totals: HashMap<String, f64> = HashMap::new();

    for record in records {
        *category_totals.entry(record.category.clone()).or_insert(0.0) += record.value;
    }

    category_totals.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_by_category() {
        let records = vec![
            Record {
                id: 1,
                name: "Item1".to_string(),
                value: 10.5,
                category: "A".to_string(),
            },
            Record {
                id: 2,
                name: "Item2".to_string(),
                value: 20.0,
                category: "B".to_string(),
            },
        ];

        let filtered = filter_by_category(&records, "A");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }

    #[test]
    fn test_calculate_average() {
        let records = vec![
            Record {
                id: 1,
                name: "Item1".to_string(),
                value: 10.0,
                category: "A".to_string(),
            },
            Record {
                id: 2,
                name: "Item2".to_string(),
                value: 20.0,
                category: "B".to_string(),
            },
        ];

        let avg = calculate_average(&records);
        assert_eq!(avg, 15.0);
    }
}use std::error::Error;
use std::fs::File;
use csv::{ReaderBuilder, Writer};

pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record { id, name, value, active }
    }
}

pub fn read_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let record: Record = result?;
        records.push(record);
    }

    Ok(records)
}

pub fn filter_active_records(records: Vec<Record>) -> Vec<Record> {
    records.into_iter().filter(|r| r.active).collect()
}

pub fn calculate_total_value(records: &[Record]) -> f64 {
    records.iter().map(|r| r.value).sum()
}

pub fn write_filtered_csv(output_path: &str, records: &[Record]) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let mut wtr = Writer::from_writer(file);

    for record in records {
        wtr.serialize(record)?;
    }

    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_active_records() {
        let records = vec![
            Record::new(1, "Alice".to_string(), 100.0, true),
            Record::new(2, "Bob".to_string(), 200.0, false),
            Record::new(3, "Charlie".to_string(), 300.0, true),
        ];

        let filtered = filter_active_records(records);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }

    #[test]
    fn test_calculate_total_value() {
        let records = vec![
            Record::new(1, "Test1".to_string(), 50.5, true),
            Record::new(2, "Test2".to_string(), 75.2, true),
        ];

        let total = calculate_total_value(&records);
        assert!((total - 125.7).abs() < 0.001);
    }
}