
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut records = Vec::new();
        
        for line_result in reader.lines() {
            let line = line_result?;
            if line.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<String> = line
                .split(self.delimiter)
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
    
    pub fn get_column_count(&self, records: &[Vec<String>]) -> Option<usize> {
        if records.is_empty() {
            return None;
        }
        
        let first_len = records[0].len();
        if records.iter().all(|r| r.len() == first_len) {
            Some(first_len)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_process_valid_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        assert_eq!(result[1], vec!["Alice", "25", "London"]);
    }
    
    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new("dummy.csv", ',');
        
        let valid_record = vec!["test".to_string(), "data".to_string()];
        assert!(processor.validate_record(&valid_record));
        
        let invalid_record = vec!["".to_string(), "data".to_string()];
        assert!(!processor.validate_record(&invalid_record));
    }
    
    #[test]
    fn test_get_column_count() {
        let processor = DataProcessor::new("dummy.csv", ',');
        
        let uniform_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];
        
        assert_eq!(processor.get_column_count(&uniform_records), Some(2));
        
        let irregular_records = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string()],
        ];
        
        assert_eq!(processor.get_column_count(&irregular_records), None);
    }
}