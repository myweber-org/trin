
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
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_number, line) in lines {
            let line = line?;
            let fields: Vec<String> = line.split(self.delimiter).map(|s| s.trim().to_string()).collect();
            
            if fields.iter().any(|field| field.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_number + 1).into());
            }
            
            records.push(fields);
        }

        if records.is_empty() {
            return Err("No valid data records found".into());
        }

        Ok(records)
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        if data.is_empty() {
            return Err("Empty dataset".into());
        }

        if column_index >= data[0].len() {
            return Err("Column index out of bounds".into());
        }

        let mut values = Vec::new();
        for row in data {
            if let Ok(value) = row[column_index].parse::<f64>() {
                values.push(value);
            } else {
                return Err(format!("Invalid numeric value in column {}", column_index).into());
            }
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / values.len() as f64;
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
    fn test_process_csv_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_csv(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![
            vec!["10.5".to_string(), "20.0".to_string()],
            vec!["15.5".to_string(), "25.0".to_string()],
            vec!["12.0".to_string(), "30.0".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let (mean, std_dev) = processor.calculate_statistics(&data, 0).unwrap();
        
        assert!((mean - 12.666).abs() < 0.001);
        assert!((std_dev - 2.054).abs() < 0.001);
    }
}