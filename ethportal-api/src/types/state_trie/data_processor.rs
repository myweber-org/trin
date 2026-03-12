
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
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
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                continue;
            }
            
            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();
            let valid = parts[3].parse::<bool>().unwrap_or(false);
            
            let record = DataRecord {
                id,
                value,
                category,
                valid,
            };
            
            self.records.push(record);
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|r| r.valid)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        let valid_records: Vec<&DataRecord> = self.records
            .iter()
            .filter(|r| r.valid)
            .collect();
        
        if valid_records.is_empty() {
            return None;
        }
        
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        Some(sum / valid_records.len() as f64)
    }

    pub fn group_by_category(&self) -> std::collections::HashMap<String, Vec<DataRecord>> {
        let mut groups = std::collections::HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record.clone());
        }
        
        groups
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_records(&self) -> &Vec<DataRecord> {
        &self.records
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
        assert_eq!(processor.count_records(), 0);
    }

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,valid").unwrap();
        writeln!(temp_file, "1,10.5,CategoryA,true").unwrap();
        writeln!(temp_file, "2,20.3,CategoryB,false").unwrap();
        writeln!(temp_file, "3,15.7,CategoryA,true").unwrap();
        
        let mut processor = DataProcessor::new();
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 3);
    }

    #[test]
    fn test_filter_valid() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord {
            id: 1,
            value: 10.0,
            category: "Test".to_string(),
            valid: true,
        });
        processor.records.push(DataRecord {
            id: 2,
            value: 20.0,
            category: "Test".to_string(),
            valid: false,
        });
        
        let valid = processor.filter_valid();
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0].id, 1);
    }

    #[test]
    fn test_calculate_average() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord {
            id: 1,
            value: 10.0,
            category: "Test".to_string(),
            valid: true,
        });
        processor.records.push(DataRecord {
            id: 2,
            value: 20.0,
            category: "Test".to_string(),
            valid: true,
        });
        processor.records.push(DataRecord {
            id: 3,
            value: 30.0,
            category: "Test".to_string(),
            valid: false,
        });
        
        let average = processor.calculate_average();
        assert!(average.is_some());
        assert_eq!(average.unwrap(), 15.0);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line.split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64)> {
        let numeric_values: Vec<f64> = records.iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|value| value.parse::<f64>().ok())
            .collect();

        if numeric_values.is_empty() {
            return None;
        }

        let sum: f64 = numeric_values.iter().sum();
        let count = numeric_values.len() as f64;
        let mean = sum / count;

        let variance: f64 = numeric_values.iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>() / count;

        Some((mean, variance.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert!(processor.validate_record(&records[0]));
        
        let stats = processor.calculate_statistics(&records, 2);
        assert!(stats.is_some());
        
        let (mean, std_dev) = stats.unwrap();
        assert!((mean - 50000.0).abs() < 0.1);
        assert!(std_dev > 0.0);
    }
}