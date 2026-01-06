
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub valid: bool,
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
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            // Skip header and empty lines
            if line_num == 0 || line.trim().is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 3 {
                continue;
            }
            
            let id = parts[0].parse::<u32>().unwrap_or(0);
            let value = parts[1].parse::<f64>().unwrap_or(0.0);
            let category = parts[2].to_string();
            let valid = value > 0.0 && !category.is_empty();
            
            self.records.push(DataRecord {
                id,
                value,
                category,
                valid,
            });
            
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_valid(&self) -> Vec<&DataRecord> {
        self.records.iter().filter(|r| r.valid).collect()
    }

    pub fn calculate_average(&self) -> f64 {
        let valid_records: Vec<&DataRecord> = self.filter_valid();
        if valid_records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = valid_records.iter().map(|r| r.value).sum();
        sum / valid_records.len() as f64
    }

    pub fn group_by_category(&self) -> Vec<(String, f64)> {
        use std::collections::HashMap;
        
        let mut categories: HashMap<String, (f64, usize)> = HashMap::new();
        
        for record in &self.records {
            if record.valid {
                let entry = categories.entry(record.category.clone())
                    .or_insert((0.0, 0));
                entry.0 += record.value;
                entry.1 += 1;
            }
        }
        
        let mut result: Vec<(String, f64)> = categories
            .into_iter()
            .map(|(category, (total, count))| (category, total / count as f64))
            .collect();
        
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    pub fn get_statistics(&self) -> (usize, usize, f64, f64) {
        let total = self.records.len();
        let valid_count = self.filter_valid().len();
        let avg_value = self.calculate_average();
        
        let max_value = self.records
            .iter()
            .filter(|r| r.valid)
            .map(|r| r.value)
            .fold(0.0, f64::max);
        
        (total, valid_count, avg_value, max_value)
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
        
        // Create test CSV data
        let csv_data = "id,value,category\n1,10.5,TypeA\n2,0.0,TypeB\n3,15.2,TypeA\n4,8.7,TypeC\n";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4);
        
        let (total, valid, avg, max) = processor.get_statistics();
        assert_eq!(total, 4);
        assert_eq!(valid, 3);
        assert!((avg - 11.466666666666667).abs() < 0.0001);
        assert!((max - 15.2).abs() < 0.0001);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 3);
        
        let valid_records = processor.filter_valid();
        assert_eq!(valid_records.len(), 3);
    }
}