
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct DataRecord {
    id: u32,
    value: f64,
    category: String,
    valid: bool,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String) -> Self {
        let valid = value >= 0.0 && !category.is_empty();
        DataRecord {
            id,
            value,
            category,
            valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
    total_value: f64,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            total_value: 0.0,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 3 {
                continue;
            }

            let id = match parts[0].parse::<u32>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let value = match parts[1].parse::<f64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let category = parts[2].trim().to_string();

            let record = DataRecord::new(id, value, category);
            if record.is_valid() {
                self.total_value += record.get_value();
            }
            self.records.push(record);
        }

        Ok(())
    }

    pub fn get_valid_count(&self) -> usize {
        self.records.iter().filter(|r| r.is_valid()).count()
    }

    pub fn get_total_value(&self) -> f64 {
        self.total_value
    }

    pub fn get_average_value(&self) -> Option<f64> {
        let valid_count = self.get_valid_count();
        if valid_count > 0 {
            Some(self.total_value / valid_count as f64)
        } else {
            None
        }
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|r| r.is_valid() && r.category == category)
            .collect()
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
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,10.5,category_a").unwrap();
        writeln!(temp_file, "2,-5.0,category_b").unwrap();
        writeln!(temp_file, "3,20.0,category_a").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "4,15.0,category_b").unwrap();

        processor.load_from_file(temp_file.path()).unwrap();

        assert_eq!(processor.get_valid_count(), 3);
        assert_eq!(processor.get_total_value(), 45.5);
        assert_eq!(processor.get_average_value(), Some(15.166666666666666));
        
        let category_a_records = processor.filter_by_category("category_a");
        assert_eq!(category_a_records.len(), 2);
    }
}