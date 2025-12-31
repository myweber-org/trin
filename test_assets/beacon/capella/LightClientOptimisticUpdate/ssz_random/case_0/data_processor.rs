use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    value: f64,
    category: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    pub fn calculate_mean(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .collect()
    }

    pub fn get_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        // Create temporary CSV file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category").unwrap();
        writeln!(temp_file, "1,10.5,A").unwrap();
        writeln!(temp_file, "2,20.3,B").unwrap();
        writeln!(temp_file, "3,15.7,A").unwrap();
        
        let file_path = temp_file.path().to_str().unwrap();
        
        // Test loading data
        assert!(processor.load_from_csv(file_path).is_ok());
        assert_eq!(processor.count_records(), 3);
        
        // Test calculations
        assert_eq!(processor.calculate_mean(), Some(15.5));
        
        // Test filtering
        let category_a = processor.filter_by_category("A");
        assert_eq!(category_a.len(), 2);
        
        // Test max value
        let max_record = processor.get_max_value().unwrap();
        assert_eq!(max_record.id, 2);
        assert_eq!(max_record.value, 20.3);
    }
}
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
            
            if line.is_empty() {
                continue;
            }

            if self.has_header && line_number == 0 {
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

    fn validate_record(&self, fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|field| !field.is_empty())
    }

    pub fn calculate_statistics(&self, data: &[Vec<String>], column_index: usize) -> Option<(f64, f64, f64)> {
        if data.is_empty() {
            return None;
        }

        let mut values = Vec::new();
        for record in data {
            if column_index < record.len() {
                if let Ok(value) = record[column_index].parse::<f64>() {
                    values.push(value);
                }
            }
        }

        if values.is_empty() {
            return None;
        }

        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        let mean = sum / count;

        let variance: f64 = values.iter()
            .map(|x| (x - mean).powi(2))
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
    fn test_data_processor() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "John,30,50000").unwrap();
        writeln!(temp_file, "Jane,25,45000").unwrap();
        writeln!(temp_file, "Bob,35,60000").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path());
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], vec!["John", "30", "50000"]);

        let stats = processor.calculate_statistics(&data, 1);
        assert!(stats.is_some());
        let (mean, _, _) = stats.unwrap();
        assert!((mean - 30.0).abs() < 0.001);
    }
}