use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u64,
    pub values: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u64, values: Vec<f64>) -> Self {
        Self {
            id,
            values,
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0 {
            return Err("Invalid record ID".to_string());
        }

        if self.values.is_empty() {
            return Err("Empty values array".to_string());
        }

        for value in &self.values {
            if value.is_nan() || value.is_infinite() {
                return Err("Invalid numeric value detected".to_string());
            }
        }

        Ok(())
    }

    pub fn transform(&mut self, factor: f64) {
        for value in &mut self.values {
            *value *= factor;
        }
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

pub fn process_records(records: &mut [DataRecord], factor: f64) -> Result<Vec<DataRecord>, String> {
    let mut processed = Vec::new();

    for record in records {
        record.validate()?;
        let mut transformed = record.clone();
        transformed.transform(factor);
        processed.push(transformed);
    }

    Ok(processed)
}

pub fn calculate_statistics(records: &[DataRecord]) -> HashMap<String, f64> {
    let mut stats = HashMap::new();

    if records.is_empty() {
        return stats;
    }

    let total_values: usize = records.iter().map(|r| r.values.len()).sum();
    let all_values: Vec<f64> = records.iter().flat_map(|r| r.values.clone()).collect();

    let sum: f64 = all_values.iter().sum();
    let count = all_values.len() as f64;

    stats.insert("total_records".to_string(), records.len() as f64);
    stats.insert("total_values".to_string(), total_values as f64);
    stats.insert("average".to_string(), sum / count);

    if let Some(min) = all_values.iter().copied().reduce(f64::min) {
        stats.insert("minimum".to_string(), min);
    }

    if let Some(max) = all_values.iter().copied().reduce(f64::max) {
        stats.insert("maximum".to_string(), max);
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, vec![1.0, 2.0]);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_record_transformation() {
        let mut record = DataRecord::new(1, vec![1.0, 2.0, 3.0]);
        record.transform(2.0);
        assert_eq!(record.values, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_process_records() {
        let mut records = vec![
            DataRecord::new(1, vec![1.0, 2.0]),
            DataRecord::new(2, vec![3.0, 4.0]),
        ];

        let result = process_records(&mut records, 2.0);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed[0].values, vec![2.0, 4.0]);
        assert_eq!(processed[1].values, vec![6.0, 8.0]);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
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
        let reader = BufReader::new(file);
        
        let mut count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }
            
            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };
            
            let category = parts[2].to_string();
            let valid = value >= 0.0 && value <= 1000.0;
            
            self.records.push(DataRecord {
                id,
                value,
                category,
                valid,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        
        if valid_records.is_empty() {
            return None;
        }
        
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<&DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            if record.valid {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn count_valid(&self) -> usize {
        self.records.iter().filter(|r| r.valid).count()
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
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,TypeA").unwrap();
        writeln!(temp_file, "2,200.0,TypeB").unwrap();
        writeln!(temp_file, "3,1500.0,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.count_records(), 3);
        assert_eq!(processor.count_valid(), 2);
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert_eq!(average.unwrap(), 150.25);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("TypeA").unwrap().len(), 1);
        assert_eq!(groups.get("TypeB").unwrap().len(), 1);
    }
}