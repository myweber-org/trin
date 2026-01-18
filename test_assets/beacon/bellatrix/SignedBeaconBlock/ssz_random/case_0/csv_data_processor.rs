
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

impl CsvRecord {
    pub fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Self {
            id,
            name,
            value,
            category,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
            && self.value >= 0.0
            && !self.category.trim().is_empty()
            && self.id > 0
    }

    pub fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let id = parts[0].parse::<u32>().unwrap_or(0);
                let name = parts[1].to_string();
                let value = parts[2].parse::<f64>().unwrap_or(0.0);
                let category = parts[3].to_string();

                let record = CsvRecord::new(id, name, value, category);
                if record.is_valid() {
                    self.records.push(record);
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn get_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }

    pub fn apply_value_transformation(&mut self, multiplier: f64) {
        for record in &mut self.records {
            record.transform_value(multiplier);
        }
    }

    pub fn get_records(&self) -> &[CsvRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = CsvRecord::new(1, "Test".to_string(), 10.5, "A".to_string());
        assert!(valid_record.is_valid());

        let invalid_record = CsvRecord::new(0, "".to_string(), -5.0, "".to_string());
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = CsvRecord::new(1, "Test".to_string(), 10.0, "A".to_string());
        record.transform_value(2.0);
        assert_eq!(record.value, 20.0);
    }

    #[test]
    fn test_processor_calculations() {
        let mut processor = CsvProcessor::new();
        
        processor.records.push(CsvRecord::new(1, "Item1".to_string(), 10.0, "A".to_string()));
        processor.records.push(CsvRecord::new(2, "Item2".to_string(), 20.0, "B".to_string()));
        processor.records.push(CsvRecord::new(3, "Item3".to_string(), 30.0, "A".to_string()));

        assert_eq!(processor.calculate_total_value(), 60.0);
        assert_eq!(processor.get_average_value(), 20.0);
        
        let category_a_records = processor.filter_by_category("A");
        assert_eq!(category_a_records.len(), 2);
    }
}