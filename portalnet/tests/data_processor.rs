
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Self {
        DataRecord {
            id,
            value,
            category: category.to_string(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value >= 0.0 && !self.category.is_empty()
    }

    pub fn calculate_metric(&self) -> f64 {
        self.value * (self.id as f64).sqrt()
    }
}

pub fn load_csv_data(file_path: &str) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let path = Path::new(file_path);
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut records = Vec::new();

    for result in rdr.deserialize() {
        let raw_record: (u32, f64, String) = result?;
        let record = DataRecord::new(raw_record.0, raw_record.1, &raw_record.2);
        if record.is_valid() {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn process_records(records: &[DataRecord]) -> (f64, usize) {
    let total: f64 = records.iter().map(|r| r.calculate_metric()).sum();
    let count = records.len();
    (total, count)
}

pub fn filter_by_category(records: Vec<DataRecord>, category: &str) -> Vec<DataRecord> {
    records
        .into_iter()
        .filter(|r| r.category == category)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_record() {
        let record = DataRecord::new(1, 10.5, "A");
        assert!(record.is_valid());
        assert_eq!(record.calculate_metric(), 10.5);
    }

    #[test]
    fn test_invalid_record() {
        let record = DataRecord::new(0, -5.0, "");
        assert!(!record.is_valid());
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            DataRecord::new(1, 10.0, "A"),
            DataRecord::new(2, 20.0, "B"),
            DataRecord::new(3, 30.0, "A"),
        ];
        
        let filtered = filter_by_category(records, "A");
        assert_eq!(filtered.len(), 2);
    }
}