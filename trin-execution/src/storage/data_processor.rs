
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
        
        for line in reader.lines() {
            let line_content = line?;
            if line_content.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<String> = line_content
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if self.validate_record(&fields) {
                records.push(fields);
            } else {
                eprintln!("Warning: Invalid record skipped: {}", line_content);
            }
        }
        
        Ok(records)
    }
    
    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|f| !f.is_empty())
    }
    
    pub fn calculate_statistics(&self, column_index: usize) -> Result<(f64, f64, f64), Box<dyn Error>> {
        let records = self.process()?;
        
        let mut values = Vec::new();
        for record in records {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }
        
        if values.is_empty() {
            return Err("No valid numeric data found".into());
        }
        
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = values.iter()
            .map(|x| (x - mean).powi(2))
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
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,55000").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["name", "age", "salary"]);
        assert_eq!(result[1], vec!["Alice", "30", "50000"]);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "value").unwrap();
        writeln!(temp_file, "10.5").unwrap();
        writeln!(temp_file, "20.3").unwrap();
        writeln!(temp_file, "15.7").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let (mean, variance, std_dev) = processor.calculate_statistics(0).unwrap();
        
        let expected_mean = (10.5 + 20.3 + 15.7) / 3.0;
        let expected_variance = ((10.5 - expected_mean).powi(2) + 
                                (20.3 - expected_mean).powi(2) + 
                                (15.7 - expected_mean).powi(2)) / 3.0;
        let expected_std_dev = expected_variance.sqrt();
        
        assert!((mean - expected_mean).abs() < 0.0001);
        assert!((variance - expected_variance).abs() < 0.0001);
        assert!((std_dev - expected_std_dev).abs() < 0.0001);
    }
}