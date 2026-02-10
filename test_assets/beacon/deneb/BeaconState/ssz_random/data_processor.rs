
use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn validate_records(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.value >= 0.0 && !r.name.is_empty())
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&Record>> {
        let mut grouped = std::collections::HashMap::new();

        for record in &self.records {
            grouped
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }

        grouped
    }

    pub fn get_statistics(&self) -> Statistics {
        let valid_records = self.validate_records();
        let average = self.calculate_average().unwrap_or(0.0);
        let max_value = self
            .records
            .iter()
            .map(|r| r.value)
            .fold(f64::NEG_INFINITY, f64::max);

        Statistics {
            total_records: self.records.len(),
            valid_records: valid_records.len(),
            average_value: average,
            maximum_value: max_value,
        }
    }
}

pub struct Statistics {
    pub total_records: usize,
    pub valid_records: usize,
    pub average_value: f64,
    pub maximum_value: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,20.3,Category2").unwrap();
        writeln!(temp_file, "3,ItemC,15.7,Category1").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);

        let stats = processor.get_statistics();
        assert_eq!(stats.total_records, 3);
        assert_eq!(stats.valid_records, 3);
        assert!((stats.average_value - 15.5).abs() < 0.1);
        assert!((stats.maximum_value - 20.3).abs() < 0.1);

        let grouped = processor.group_by_category();
        assert_eq!(grouped.get("Category1").unwrap().len(), 2);
        assert_eq!(grouped.get("Category2").unwrap().len(), 1);
    }
}