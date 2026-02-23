
use std::error::Error;
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

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if fields.iter().any(|f| f.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid records found in file".into());
        }

        Ok(records)
    }

    pub fn validate_records(&self, records: &[Vec<String>]) -> Result<(), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to validate".into());
        }

        let expected_len = records[0].len();
        
        for (idx, record) in records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!("Record {} has {} fields, expected {}", 
                    idx + 1, record.len(), expected_len).into());
            }

            for (field_idx, field) in record.iter().enumerate() {
                if field.contains('\n') || field.contains('\r') {
                    return Err(format!("Newline character found in record {}, field {}", 
                        idx + 1, field_idx + 1).into());
                }
            }
        }

        Ok(())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64, f64), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records for statistics calculation".into());
        }

        if column_index >= records[0].len() {
            return Err(format!("Column index {} out of bounds", column_index).into());
        }

        let mut values = Vec::new();
        for record in records {
            if let Ok(value) = record[column_index].parse::<f64>() {
                values.push(value);
            } else {
                return Err(format!("Non-numeric value in column {}: {}", 
                    column_index, record[column_index]).into());
            }
        }

        if values.is_empty() {
            return Err("No numeric values found in specified column".into());
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        Ok((mean, variance, std_dev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,25,New York").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec!["John", "25", "New York"]);
    }

    #[test]
    fn test_validate_records() {
        let processor = DataProcessor::new(',', false);
        let valid_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        assert!(processor.validate_records(&valid_records).is_ok());
    }

    #[test]
    fn test_calculate_statistics() {
        let processor = DataProcessor::new(',', false);
        let records = vec![
            vec!["10.5".to_string(), "test".to_string()],
            vec!["20.5".to_string(), "test".to_string()],
            vec!["30.5".to_string(), "test".to_string()],
        ];
        
        let stats = processor.calculate_statistics(&records, 0).unwrap();
        assert!((stats.0 - 20.5).abs() < 0.001);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
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
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
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
            
            if !self.validate_record(id, value, &category) {
                continue;
            }
            
            self.records.push(DataRecord { id, value, category });
            count += 1;
        }
        
        Ok(count)
    }
    
    fn validate_record(&self, id: u32, value: f64, category: &str) -> bool {
        if id == 0 {
            return false;
        }
        
        if value < 0.0 || value > 10000.0 {
            return false;
        }
        
        if category.is_empty() || category.len() > 50 {
            return false;
        }
        
        true
    }
    
    pub fn calculate_statistics(&self) -> (f64, f64, f64) {
        if self.records.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        let count = self.records.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = self.records.iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
    
    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records.iter()
            .filter(|r| r.category == category)
            .collect()
    }
    
    pub fn get_total_records(&self) -> usize {
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
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,100.5,TypeA").unwrap();
        writeln!(temp_file, "2,200.3,TypeB").unwrap();
        writeln!(temp_file, "3,150.8,TypeA").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
        assert_eq!(processor.get_total_records(), 3);
        
        let stats = processor.calculate_statistics();
        assert!(stats.0 > 0.0);
        
        let filtered = processor.filter_by_category("TypeA");
        assert_eq!(filtered.len(), 2);
    }
    
    #[test]
    fn test_validation() {
        let processor = DataProcessor::new();
        
        assert!(processor.validate_record(1, 50.0, "Valid"));
        assert!(!processor.validate_record(0, 50.0, "Valid"));
        assert!(!processor.validate_record(1, -10.0, "Valid"));
        assert!(!processor.validate_record(1, 50000.0, "Valid"));
        assert!(!processor.validate_record(1, 50.0, ""));
    }
}