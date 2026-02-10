
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: &str) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        if category.trim().is_empty() {
            return Err("Category cannot be empty".to_string());
        }
        
        Ok(Self {
            id,
            value,
            category: category.to_string(),
        })
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }
    
    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                return Err(format!("Invalid CSV format at line {}", line_num + 1).into());
            }
            
            let id = parts[0].parse::<u32>()
                .map_err(|e| format!("Invalid ID at line {}: {}", line_num + 1, e))?;
            
            let value = parts[1].parse::<f64>()
                .map_err(|e| format!("Invalid value at line {}: {}", line_num + 1, e))?;
            
            let record = DataRecord::new(id, value, parts[2])?;
            self.records.push(record);
            count += 1;
        }
        
        Ok(count)
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
    
    pub fn get_records(&self) -> &[DataRecord] {
        &self.records
    }
    
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, 42.5, "test").unwrap();
        assert_eq!(record.id, 1);
        assert_eq!(record.value, 42.5);
        assert_eq!(record.category, "test");
    }
    
    #[test]
    fn test_invalid_data_record() {
        assert!(DataRecord::new(1, -5.0, "test").is_err());
        assert!(DataRecord::new(1, 5.0, "").is_err());
    }
    
    #[test]
    fn test_csv_loading() {
        let mut csv_content = "1,42.5,category_a\n".to_string();
        csv_content.push_str("2,33.7,category_b\n");
        csv_content.push_str("3,89.1,category_a\n");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_content).unwrap();
        
        let mut processor = DataProcessor::new();
        let count = processor.load_from_csv(temp_file.path()).unwrap();
        
        assert_eq!(count, 3);
        assert_eq!(processor.get_records().len(), 3);
    }
    
    #[test]
    fn test_statistics_calculation() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "test").unwrap());
        processor.records.push(DataRecord::new(2, 20.0, "test").unwrap());
        processor.records.push(DataRecord::new(3, 30.0, "test").unwrap());
        
        let (mean, variance, std_dev) = processor.calculate_statistics();
        
        assert!((mean - 20.0).abs() < 0.001);
        assert!((variance - 66.666).abs() < 0.001);
        assert!((std_dev - 8.1649).abs() < 0.001);
    }
    
    #[test]
    fn test_category_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 10.0, "cat_a").unwrap());
        processor.records.push(DataRecord::new(2, 20.0, "cat_b").unwrap());
        processor.records.push(DataRecord::new(3, 30.0, "cat_a").unwrap());
        
        let filtered = processor.filter_by_category("cat_a");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].id, 1);
        assert_eq!(filtered[1].id, 3);
    }
}use std::error::Error;
use std::fs::File;
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

    pub fn process(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        let mut records = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let fields: Vec<String> = record.iter().map(|s| s.to_string()).collect();
            
            if Self::validate_record(&fields) {
                records.push(fields);
            } else {
                eprintln!("Warning: Skipping invalid record: {:?}", fields);
            }
        }
        
        Ok(records)
    }

    fn validate_record(fields: &[String]) -> bool {
        !fields.is_empty() && fields.iter().all(|f| !f.trim().is_empty())
    }

    pub fn calculate_statistics(data: &[Vec<String>]) -> (usize, usize) {
        let total_records = data.len();
        let total_fields = data.iter().map(|record| record.len()).sum();
        (total_records, total_fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Alice,25,London").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
        
        let stats = DataProcessor::calculate_statistics(&result);
        assert_eq!(stats, (2, 6));
    }

    #[test]
    fn test_invalid_data_skipping() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "field1,field2").unwrap();
        writeln!(temp_file, "value1,").unwrap();
        writeln!(temp_file, ",value2").unwrap();
        writeln!(temp_file, "valid1,valid2").unwrap();
        
        let processor = DataProcessor::new(temp_file.path().to_str().unwrap());
        let result = processor.process().unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["valid1", "valid2"]);
    }
}