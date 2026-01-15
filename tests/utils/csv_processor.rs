
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn validate_file(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err("File does not exist".into());
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            line_count += 1;

            let fields: Vec<&str> = line.split(self.delimiter).collect();
            if fields.is_empty() {
                return Err(format!("Line {} is empty", index + 1).into());
            }

            if index == 0 && self.has_header {
                for (col_idx, header) in fields.iter().enumerate() {
                    if header.trim().is_empty() {
                        return Err(format!("Empty header at column {}", col_idx + 1).into());
                    }
                }
            }
        }

        if line_count == 0 {
            return Err("File is empty".into());
        }

        Ok(())
    }

    pub fn transform_column<F>(&self, file_path: &str, column_index: usize, transform_fn: F) -> Result<Vec<String>, Box<dyn Error>>
    where
        F: Fn(&str) -> String,
    {
        self.validate_file(file_path)?;

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();
        let mut is_first_line = true;

        for line in reader.lines() {
            let line = line?;
            
            if is_first_line && self.has_header {
                is_first_line = false;
                continue;
            }

            let fields: Vec<&str> = line.split(self.delimiter).collect();
            
            if column_index >= fields.len() {
                return Err(format!("Column index {} out of bounds", column_index).into());
            }

            let transformed = transform_fn(fields[column_index]);
            results.push(transformed);
        }

        Ok(results)
    }

    pub fn calculate_column_stats(&self, file_path: &str, column_index: usize) -> Result<(f64, f64, f64), Box<dyn Error>> {
        let numeric_values: Vec<f64> = self.transform_column(file_path, column_index, |value| {
            value.parse::<f64>().unwrap_or(0.0).to_string()
        })?
        .iter()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();

        if numeric_values.is_empty() {
            return Err("No valid numeric values found in column".into());
        }

        let sum: f64 = numeric_values.iter().sum();
        let count = numeric_values.len() as f64;
        let mean = sum / count;
        
        let variance: f64 = numeric_values.iter()
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
    fn test_csv_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,25,New York").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_column_transformation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age").unwrap();
        writeln!(temp_file, "Alice,30").unwrap();
        writeln!(temp_file, "Bob,25").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let results = processor.transform_column(
            temp_file.path().to_str().unwrap(),
            1,
            |age| format!("Age: {}", age)
        ).unwrap();
        
        assert_eq!(results, vec!["Age: 30", "Age: 25"]);
    }

    #[test]
    fn test_column_stats() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "value").unwrap();
        writeln!(temp_file, "10").unwrap();
        writeln!(temp_file, "20").unwrap();
        writeln!(temp_file, "30").unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let (mean, variance, std_dev) = processor.calculate_column_stats(
            temp_file.path().to_str().unwrap(),
            0
        ).unwrap();
        
        assert_eq!(mean, 20.0);
        assert_eq!(variance, 66.66666666666667);
        assert_eq!(std_dev, 8.16496580927726);
    }
}