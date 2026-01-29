
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    pub fn new(id: u32, category: String, value: f64, timestamp: String) -> Self {
        DataRecord {
            id,
            category,
            value,
            timestamp,
        }
    }
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
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self, category: Option<&str>) -> f64 {
        let filtered_records: Vec<&DataRecord> = match category {
            Some(cat) => self.filter_by_category(cat),
            None => self.records.iter().collect(),
        };

        if filtered_records.is_empty() {
            return 0.0;
        }

        let sum: f64 = filtered_records.iter().map(|r| r.value).sum();
        sum / filtered_records.len() as f64
    }

    pub fn save_filtered_results(&self, category: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let file = File::create(output_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new().has_headers(true).from_writer(writer);

        for record in filtered {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    pub fn get_statistics(&self) -> (f64, f64, f64) {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let count = values.len() as f64;
        
        if count == 0.0 {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,timestamp").unwrap();
        writeln!(temp_file, "1,electronics,150.5,2024-01-15").unwrap();
        writeln!(temp_file, "2,clothing,75.2,2024-01-16").unwrap();
        writeln!(temp_file, "3,electronics,200.0,2024-01-17").unwrap();
        writeln!(temp_file, "4,clothing,50.0,2024-01-18").unwrap();
        temp_file
    }

    #[test]
    fn test_load_and_filter() {
        let temp_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let clothing = processor.filter_by_category("clothing");
        assert_eq!(clothing.len(), 2);
    }

    #[test]
    fn test_calculate_average() {
        let temp_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        let electronics_avg = processor.calculate_average(Some("electronics"));
        assert_eq!(electronics_avg, 175.25);
        
        let overall_avg = processor.calculate_average(None);
        assert_eq!(overall_avg, 118.925);
    }

    #[test]
    fn test_statistics() {
        let temp_file = create_test_csv();
        let mut processor = DataProcessor::new();
        
        processor.load_from_csv(temp_file.path().to_str().unwrap()).unwrap();
        
        let (mean, variance, std_dev) = processor.get_statistics();
        assert_eq!(mean, 118.925);
        assert!(variance > 0.0);
        assert!(std_dev > 0.0);
    }
}