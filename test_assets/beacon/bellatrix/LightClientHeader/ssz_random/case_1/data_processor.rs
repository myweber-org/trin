
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
}

impl DataProcessor {
    pub fn new(file_path: &str) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
        }
    }

    pub fn process_csv(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut records = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            
            if !fields.is_empty() && !fields.iter().all(|f| f.is_empty()) {
                records.push(fields);
            }
        }
        
        Ok(records)
    }

    pub fn validate_data(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No data found in file".into());
        }
        
        let header_len = records[0].len();
        
        for (i, record) in records.iter().enumerate() {
            if record.len() != header_len {
                return Err(format!("Row {} has {} fields, expected {}", i + 1, record.len(), header_len).into());
            }
            
            for (j, field) in record.iter().enumerate() {
                if field.is_empty() {
                    return Err(format!("Empty field at row {}, column {}", i + 1, j + 1).into());
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64, f64), Box<dyn Error>> {
        if records.len() < 2 {
            return Err("Insufficient data for statistics".into());
        }
        
        if column_index >= records[0].len() {
            return Err("Column index out of bounds".into());
        }
        
        let mut values = Vec::new();
        
        for (i, record) in records.iter().enumerate().skip(1) {
            if let Ok(value) = record[column_index].parse::<f64>() {
                values.push(value);
            } else {
                return Err(format!("Invalid numeric value at row {}, column {}", i + 1, column_index + 1).into());
            }
        }
        
        if values.is_empty() {
            return Err("No valid numeric values found".into());
        }
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Ok((mean, std_dev, max - min))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process_csv().unwrap();
        
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["name", "age", "salary"]);
        assert_eq!(result[1], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_validate_data() {
        let records = vec![
            vec!["name".to_string(), "age".to_string()],
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];
        
        let processor = DataProcessor::new("test.csv");
        let result = processor.validate_data(&records);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["name".to_string(), "age".to_string()],
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
            vec!["Charlie".to_string(), "35".to_string()],
        ];
        
        let processor = DataProcessor::new("test.csv");
        let (mean, std_dev, range) = processor.calculate_statistics(&records, 1).unwrap();
        
        assert!((mean - 30.0).abs() < 0.001);
        assert!(std_dev > 0.0);
        assert!((range - 10.0).abs() < 0.001);
    }
}