use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        self.records.clear();

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    pub fn save_filtered_results(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in filtered {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn get_statistics(&self) -> (usize, f64, f64) {
        let count = self.records.len();
        let avg = self.calculate_average();
        let max = self.records
            .iter()
            .map(|record| record.value)
            .fold(f64::NEG_INFINITY, f64::max);

        (count, avg, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let test_data = vec![
            DataRecord { id: 1, name: "Item A".to_string(), value: 10.5, category: "Category1".to_string() },
            DataRecord { id: 2, name: "Item B".to_string(), value: 20.3, category: "Category2".to_string() },
            DataRecord { id: 3, name: "Item C".to_string(), value: 15.7, category: "Category1".to_string() },
        ];

        processor.records = test_data;

        let filtered = processor.filter_by_category("Category1");
        assert_eq!(filtered.len(), 2);

        let avg = processor.calculate_average();
        assert!((avg - 15.5).abs() < 0.1);

        let stats = processor.get_statistics();
        assert_eq!(stats.0, 3);
    }
}