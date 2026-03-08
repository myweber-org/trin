
use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
pub struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    pub fn new(id: u32, category: &str, value: f64, active: bool) -> Self {
        DataRecord {
            id,
            category: category.to_string(),
            value,
            active,
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
        let mut rdr = Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn export_active_records(&self, output_path: &str) -> Result<(), Box<dyn Error>> {
        let active_records: Vec<&DataRecord> = self.records.iter().filter(|r| r.active).collect();

        let mut wtr = Writer::from_path(output_path)?;

        for record in active_records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();

        processor.add_record(DataRecord::new(1, "A", 10.5, true));
        processor.add_record(DataRecord::new(2, "B", 20.3, false));
        processor.add_record(DataRecord::new(3, "A", 15.7, true));

        assert_eq!(processor.get_record_count(), 3);

        let filtered = processor.filter_by_category("A");
        assert_eq!(filtered.len(), 2);

        let avg = processor.calculate_average();
        assert!(avg.is_some());
        assert!((avg.unwrap() - 15.5).abs() < 0.01);
    }

    #[test]
    fn test_csv_export() {
        let mut processor = DataProcessor::new();
        processor.add_record(DataRecord::new(1, "Test", 42.0, true));

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        let result = processor.export_active_records(path);
        assert!(result.is_ok());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers_line = lines.next()
            .ok_or("Empty CSV file")??;
        let headers: Vec<String> = headers_line
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let mut records = Vec::new();
        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            
            if fields.len() == headers.len() {
                records.push(fields);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name);
        
        match column_index {
            Some(idx) => self.records.iter()
                .filter(|record| predicate(&record[idx]))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn aggregate_numeric_column(&self, group_column: &str, value_column: &str) -> HashMap<String, f64> {
        let group_idx = self.headers.iter()
            .position(|h| h == group_column);
        let value_idx = self.headers.iter()
            .position(|h| h == value_column);

        match (group_idx, value_idx) {
            (Some(g_idx), Some(v_idx)) => {
                let mut aggregates = HashMap::new();
                for record in &self.records {
                    if let (Some(group_val), Some(value_str)) = (record.get(g_idx), record.get(v_idx)) {
                        if let Ok(value) = value_str.parse::<f64>() {
                            *aggregates.entry(group_val.clone()).or_insert(0.0) += value;
                        }
                    }
                }
                aggregates
            }
            _ => HashMap::new(),
        }
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
}

pub fn process_csv_data(path: &str) -> Result<(), Box<dyn Error>> {
    let processor = CsvProcessor::from_file(path)?;
    
    println!("Loaded {} records with columns: {:?}", 
             processor.get_record_count(), 
             processor.get_headers());
    
    let filtered = processor.filter_by_column("status", |val| val == "active");
    println!("Active records: {}", filtered.len());
    
    let aggregates = processor.aggregate_numeric_column("category", "amount");
    for (category, total) in aggregates {
        println!("Category {}: total amount {}", category, total);
    }
    
    Ok(())
}