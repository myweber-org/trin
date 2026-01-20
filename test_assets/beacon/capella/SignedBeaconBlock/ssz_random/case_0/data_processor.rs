use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

pub fn process_data_file(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let mut records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        if record.value >= 0.0 {
            records.push(record);
        }
    }

    Ok(records)
}

pub fn calculate_statistics(records: &[Record]) -> (f64, f64, usize) {
    let total: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len();
    let average = if count > 0 { total / count as f64 } else { 0.0 };
    
    let active_count = records.iter().filter(|r| r.active).count();
    
    (total, average, active_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_process_valid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,ItemA,10.5,true").unwrap();
        writeln!(temp_file, "2,ItemB,-5.0,false").unwrap();
        writeln!(temp_file, "3,ItemC,15.0,true").unwrap();
        
        let records = process_data_file(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "ItemA");
        assert_eq!(records[1].name, "ItemC");
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            Record { id: 1, name: "Test1".to_string(), value: 10.0, active: true },
            Record { id: 2, name: "Test2".to_string(), value: 20.0, active: false },
            Record { id: 3, name: "Test3".to_string(), value: 30.0, active: true },
        ];
        
        let (total, average, active_count) = calculate_statistics(&records);
        assert_eq!(total, 60.0);
        assert_eq!(average, 20.0);
        assert_eq!(active_count, 2);
    }
}
use std::error::Error;
use std::fs::File;
use std::path::Path;

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

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        let mut count = 0;
        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.validate_record(&record)?;
            self.records.push(record);
            count += 1;
        }

        Ok(count)
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn get_top_values(&self, limit: usize) -> Vec<&DataRecord> {
        let mut sorted_records: Vec<&DataRecord> = self.records.iter().collect();
        sorted_records.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
        sorted_records.truncate(limit);
        sorted_records
    }

    fn validate_record(&self, record: &DataRecord) -> Result<(), Box<dyn Error>> {
        if record.name.is_empty() {
            return Err("Record name cannot be empty".into());
        }
        if record.value < 0.0 {
            return Err("Record value cannot be negative".into());
        }
        if record.category.is_empty() {
            return Err("Record category cannot be empty".into());
        }
        Ok(())
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,category").unwrap();
        writeln!(temp_file, "1,ItemA,25.5,Category1").unwrap();
        writeln!(temp_file, "2,ItemB,30.2,Category1").unwrap();
        writeln!(temp_file, "3,ItemC,15.8,Category2").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.record_count(), 3);

        let stats = processor.calculate_statistics();
        assert!((stats.0 - 23.833333).abs() < 0.001);

        let filtered = processor.filter_by_category("Category1");
        assert_eq!(filtered.len(), 2);

        let top_values = processor.get_top_values(2);
        assert_eq!(top_values.len(), 2);
        assert_eq!(top_values[0].name, "ItemB");
    }
}