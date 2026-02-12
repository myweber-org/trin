
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

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_number + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, record: &[String]) -> bool {
        !record.is_empty() && record.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        if records.is_empty() {
            return Err("No records to process".into());
        }

        let mut values = Vec::new();
        for record in records {
            if column_index >= record.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }

            match record[column_index].parse::<f64>() {
                Ok(value) => values.push(value),
                Err(_) => return Err(format!("Invalid numeric value at column {}", column_index).into()),
            }
        }

        let sum: f64 = values.iter().sum();
        let mean = sum / values.len() as f64;
        let variance: f64 = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
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
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.0").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        writeln!(temp_file, "Charlie,35,55000.0").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec!["Alice", "30", "50000.0"]);
        
        let stats = processor.calculate_statistics(&records, 2).unwrap();
        assert!((stats.0 - 50000.0).abs() < 0.01);
    }

    #[test]
    fn test_invalid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        writeln!(temp_file, "Alice,thirty").unwrap();

        let processor = DataProcessor::new(',', true);
        let records = processor.process_file(temp_file.path()).unwrap();
        let result = processor.calculate_statistics(&records, 1);
        
        assert!(result.is_err());
    }
}