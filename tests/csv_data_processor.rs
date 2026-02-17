use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub fn read_csv_file(file_path: &str) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if index == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 4 {
            let record = CsvRecord {
                id: parts[0].parse()?,
                name: parts[1].to_string(),
                value: parts[2].parse()?,
                category: parts[3].to_string(),
            };
            records.push(record);
        }
    }

    Ok(records)
}

pub fn filter_by_category(records: &[CsvRecord], category: &str) -> Vec<&CsvRecord> {
    records
        .iter()
        .filter(|record| record.category == category)
        .collect()
}

pub fn calculate_average_value(records: &[CsvRecord]) -> f64 {
    if records.is_empty() {
        return 0.0;
    }

    let total: f64 = records.iter().map(|record| record.value).sum();
    total / records.len() as f64
}

pub fn find_max_value_record(records: &[CsvRecord]) -> Option<&CsvRecord> {
    records.iter().max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_records() -> Vec<CsvRecord> {
        vec![
            CsvRecord {
                id: 1,
                name: "Item A".to_string(),
                value: 10.5,
                category: "Electronics".to_string(),
            },
            CsvRecord {
                id: 2,
                name: "Item B".to_string(),
                value: 25.0,
                category: "Books".to_string(),
            },
            CsvRecord {
                id: 3,
                name: "Item C".to_string(),
                value: 15.75,
                category: "Electronics".to_string(),
            },
        ]
    }

    #[test]
    fn test_filter_by_category() {
        let records = create_test_records();
        let filtered = filter_by_category(&records, "Electronics");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].name, "Item A");
        assert_eq!(filtered[1].name, "Item C");
    }

    #[test]
    fn test_calculate_average_value() {
        let records = create_test_records();
        let average = calculate_average_value(&records);
        assert!((average - 17.08333).abs() < 0.0001);
    }

    #[test]
    fn test_find_max_value_record() {
        let records = create_test_records();
        let max_record = find_max_value_record(&records).unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 25.0);
    }
}