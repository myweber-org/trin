
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub value: f64,
    pub category: String,
    pub timestamp: u64,
}

impl DataRecord {
    pub fn new(id: u32, value: f64, category: String, timestamp: u64) -> Self {
        Self {
            id,
            value,
            category,
            timestamp,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.id > 0 && self.value.is_finite() && !self.category.is_empty()
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
            if line_num == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
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

            let category = parts[2].to_string();
            let timestamp = match parts[3].parse::<u64>() {
                Ok(val) => val,
                Err(_) => continue,
            };

            let record = DataRecord::new(id, value, category, timestamp);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn get_statistics(&self) -> Statistics {
        let values: Vec<f64> = self.records.iter().map(|r| r.value).collect();
        let count = values.len();

        if count == 0 {
            return Statistics::empty();
        }

        let sum: f64 = values.iter().sum();
        let avg = sum / count as f64;
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let variance: f64 = values.iter().map(|&v| (v - avg).powi(2)).sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Statistics {
            count,
            average: avg,
            minimum: min,
            maximum: max,
            standard_deviation: std_dev,
        }
    }

    pub fn sort_by_value(&mut self) {
        self.records.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct Statistics {
    pub count: usize,
    pub average: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub standard_deviation: f64,
}

impl Statistics {
    pub fn empty() -> Self {
        Self {
            count: 0,
            average: 0.0,
            minimum: 0.0,
            maximum: 0.0,
            standard_deviation: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_record_validation() {
        let valid_record = DataRecord::new(1, 42.5, "test".to_string(), 1234567890);
        assert!(valid_record.is_valid());

        let invalid_id = DataRecord::new(0, 42.5, "test".to_string(), 1234567890);
        assert!(!invalid_id.is_valid());

        let invalid_value = DataRecord::new(1, f64::NAN, "test".to_string(), 1234567890);
        assert!(!invalid_value.is_valid());

        let empty_category = DataRecord::new(1, 42.5, "".to_string(), 1234567890);
        assert!(!empty_category.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert!(processor.is_empty());

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,category,timestamp").unwrap();
        writeln!(temp_file, "1,10.5,category_a,1000").unwrap();
        writeln!(temp_file, "2,20.5,category_b,2000").unwrap();
        writeln!(temp_file, "3,30.5,category_a,3000").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.len(), 3);

        let category_a = processor.filter_by_category("category_a");
        assert_eq!(category_a.len(), 2);

        let avg = processor.calculate_average().unwrap();
        assert!((avg - 20.5).abs() < f64::EPSILON);

        let stats = processor.get_statistics();
        assert_eq!(stats.count, 3);
        assert!((stats.minimum - 10.5).abs() < f64::EPSILON);
        assert!((stats.maximum - 30.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sorting() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, 30.0, "test".to_string(), 1000));
        processor.records.push(DataRecord::new(2, 10.0, "test".to_string(), 2000));
        processor.records.push(DataRecord::new(3, 20.0, "test".to_string(), 3000));

        processor.sort_by_value();
        assert!((processor.records[0].value - 10.0).abs() < f64::EPSILON);
        assert!((processor.records[1].value - 20.0).abs() < f64::EPSILON);
        assert!((processor.records[2].value - 30.0).abs() < f64::EPSILON);
    }
}