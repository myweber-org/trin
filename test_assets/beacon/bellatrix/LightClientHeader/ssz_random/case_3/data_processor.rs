use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct DataSet {
    values: Vec<f64>,
}

impl DataSet {
    pub fn new() -> Self {
        DataSet { values: Vec::new() }
    }

    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        let mut values = Vec::new();

        for result in rdr.records() {
            let record = result?;
            for field in record.iter() {
                if let Ok(num) = field.parse::<f64>() {
                    values.push(num);
                }
            }
        }

        Ok(DataSet { values })
    }

    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        let sum: f64 = self.values.iter().sum();
        Some(sum / self.values.len() as f64)
    }

    pub fn calculate_standard_deviation(&self) -> Option<f64> {
        if self.values.len() < 2 {
            return None;
        }
        
        let mean = self.calculate_mean()?;
        let variance: f64 = self.values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (self.values.len() - 1) as f64;
        
        Some(variance.sqrt())
    }

    pub fn get_summary(&self) -> DataSummary {
        DataSummary {
            count: self.values.len(),
            mean: self.calculate_mean(),
            std_dev: self.calculate_standard_deviation(),
            min: self.values.iter().copied().reduce(f64::min),
            max: self.values.iter().copied().reduce(f64::max),
        }
    }
}

pub struct DataSummary {
    pub count: usize,
    pub mean: Option<f64>,
    pub std_dev: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl std::fmt::Display for DataSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Data Summary:")?;
        writeln!(f, "  Count: {}", self.count)?;
        
        if let Some(mean) = self.mean {
            writeln!(f, "  Mean: {:.4}", mean)?;
        }
        
        if let Some(std_dev) = self.std_dev {
            writeln!(f, "  Standard Deviation: {:.4}", std_dev)?;
        }
        
        if let Some(min) = self.min {
            writeln!(f, "  Minimum: {:.4}", min)?;
        }
        
        if let Some(max) = self.max {
            writeln!(f, "  Maximum: {:.4}", max)?;
        }
        
        Ok(())
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

    pub fn calculate_statistics(&self, records: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
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
        
        let std_dev = variance.sqrt();

        Some((mean, variance, std_dev))
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
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        
        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        assert!(processor.validate_record(&["test".to_string(), "123".to_string()]));
        assert!(!processor.validate_record(&[]));
        assert!(!processor.validate_record(&["".to_string(), "data".to_string()]));
    }

    #[test]
    fn test_calculate_statistics() {
        let records = vec![
            vec!["10.5".to_string()],
            vec!["20.0".to_string()],
            vec!["15.5".to_string()],
        ];
        
        let processor = DataProcessor::new(',', false);
        let stats = processor.calculate_statistics(&records, 0).unwrap();
        
        assert!((stats.0 - 15.333).abs() < 0.001);
        assert!((stats.1 - 16.222).abs() < 0.001);
        assert!((stats.2 - 4.027).abs() < 0.001);
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

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 && self.has_header {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !self.validate_record(&fields) {
                return Err(format!("Invalid record at line {}", line_num + 1).into());
            }

            records.push(fields);
        }

        Ok(records)
    }

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Result<(f64, f64), Box<dyn Error>> {
        let mut values = Vec::new();

        for record in data {
            if column_index >= record.len() {
                return Err("Column index out of bounds".into());
            }

            if let Ok(value) = record[column_index].parse::<f64>() {
                values.push(value);
            } else {
                return Err(format!("Invalid numeric value at column {}", column_index).into());
            }
        }

        if values.is_empty() {
            return Err("No valid numeric values found".into());
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;

        Ok((mean, variance.sqrt()))
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
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,95.5").unwrap();
        writeln!(temp_file, "Bob,30,87.2").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "25", "95.5"]);
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![
            vec!["10.0".to_string(), "20.0".to_string()],
            vec!["20.0".to_string(), "30.0".to_string()],
            vec!["30.0".to_string(), "40.0".to_string()],
        ];

        let processor = DataProcessor::new(',', false);
        let (mean, std_dev) = processor.calculate_statistics(&data, 0).unwrap();

        assert!((mean - 20.0).abs() < 0.001);
        assert!((std_dev - 8.1649).abs() < 0.001);
    }

    #[test]
    fn test_validate_record() {
        let processor = DataProcessor::new(',', false);
        
        assert!(processor.validate_record(&["a".to_string(), "b".to_string()]));
        assert!(!processor.validate_record(&[]));
        assert!(!processor.validate_record(&["".to_string(), "b".to_string()]));
    }
}