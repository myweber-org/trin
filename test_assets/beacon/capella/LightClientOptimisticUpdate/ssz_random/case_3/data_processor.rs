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

    pub fn process_csv<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
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

    pub fn validate_numeric_fields(&self, data: &[Vec<String>], column_index: usize) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut numeric_values = Vec::new();
        
        for (row_index, row) in data.iter().enumerate() {
            if column_index < row.len() {
                match row[column_index].parse::<f64>() {
                    Ok(value) => numeric_values.push(value),
                    Err(_) => return Err(format!("Invalid numeric value at row {}, column {}: '{}'", 
                        row_index + 1, column_index, row[column_index]).into()),
                }
            } else {
                return Err(format!("Column index {} out of bounds at row {}", column_index, row_index + 1).into());
            }
        }
        
        Ok(numeric_values)
    }

    pub fn calculate_statistics(&self, numbers: &[f64]) -> (f64, f64, f64) {
        if numbers.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = numbers.iter().sum();
        let mean = sum / numbers.len() as f64;
        
        let variance: f64 = numbers.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / numbers.len() as f64;
        
        let std_dev = variance.sqrt();
        
        (mean, variance, std_dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000.5").unwrap();
        writeln!(temp_file, "Bob,25,45000.0").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "50000.5"]);
    }

    #[test]
    fn test_numeric_validation() {
        let data = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.0".to_string(), "25.5".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let numbers = processor.validate_numeric_fields(&data, 0).unwrap();
        
        assert_eq!(numbers, vec![10.5, 15.0]);
    }

    #[test]
    fn test_statistics_calculation() {
        let numbers = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let processor = DataProcessor::new(',', false);
        let (mean, variance, std_dev) = processor.calculate_statistics(&numbers);
        
        assert_eq!(mean, 3.0);
        assert_eq!(variance, 2.0);
        assert_eq!(std_dev, 2.0_f64.sqrt());
    }
}