
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct DataProcessor {
    buffer_size: usize,
}

impl DataProcessor {
    pub fn new(buffer_size: usize) -> Self {
        DataProcessor { buffer_size }
    }

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::with_capacity(self.buffer_size, file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let mut records = Vec::new();
        for line in content.lines() {
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), &'static str> {
        if records.is_empty() {
            return Err("No valid records found");
        }

        let expected_columns = records[0].len();
        for (index, record) in records.iter().enumerate() {
            if record.len() != expected_columns {
                return Err(&format!("Record {} has {} columns, expected {}", 
                    index, record.len(), expected_columns));
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64), &'static str> {
        let mut values = Vec::new();
        
        for record in records.iter().skip(1) {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }

        if values.is_empty() {
            return Err("No numeric values found in specified column");
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        
        let std_dev = variance.sqrt();

        Ok((mean, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let processor = DataProcessor::new(1024);
        let mut temp_file = NamedTempFile::new().unwrap();
        
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        
        let result = processor.process_csv(temp_file.path());
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[1][0], "Alice");
    }

    #[test]
    fn test_validate_records() {
        let processor = DataProcessor::new(1024);
        let valid_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        let invalid_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string()],
        ];
        
        assert!(processor.validate_records(&valid_records).is_ok());
        assert!(processor.validate_records(&invalid_records).is_err());
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(1024);
        let records = vec![
            vec!["name".to_string(), "value".to_string()],
            vec!["test1".to_string(), "10.0".to_string()],
            vec!["test2".to_string(), "20.0".to_string()],
            vec!["test3".to_string(), "30.0".to_string()],
        ];
        
        let result = processor.calculate_statistics(&records, 1);
        assert!(result.is_ok());
        
        let (mean, std_dev) = result.unwrap();
        assert_eq!(mean, 20.0);
        assert!(std_dev > 0.0);
    }
}