
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Record {
            id,
            category,
            value,
            active,
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
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

    pub fn load_from_csv(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        // Skip header
        lines.next();

        for line in lines {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 4 {
                let id = parts[0].parse::<u32>()?;
                let category = parts[1].to_string();
                let value = parts[2].parse::<f64>()?;
                let active = parts[3].parse::<bool>()?;
                self.records.push(Record::new(id, category, value, active));
            }
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category() == category)
            .collect()
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.is_active())
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.records.iter().map(|r| r.value()).sum();
        sum / self.records.len() as f64
    }

    pub fn calculate_category_average(&self, category: &str) -> f64 {
        let filtered = self.filter_by_category(category);
        if filtered.is_empty() {
            return 0.0;
        }
        let sum: f64 = filtered.iter().map(|r| r.value()).sum();
        sum / filtered.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value()
                .partial_cmp(&b.value())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn find_min_value(&self) -> Option<&Record> {
        self.records.iter().min_by(|a, b| {
            a.value()
                .partial_cmp(&b.value())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_records(&self) -> &[Record] {
        &self.records
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
        writeln!(temp_file, "id,category,value,active").unwrap();
        writeln!(temp_file, "1,electronics,250.5,true").unwrap();
        writeln!(temp_file, "2,clothing,45.75,false").unwrap();
        writeln!(temp_file, "3,electronics,180.0,true").unwrap();
        writeln!(temp_file, "4,food,12.99,true").unwrap();

        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.count_records(), 4);

        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);

        let active_records = processor.filter_active();
        assert_eq!(active_records.len(), 3);

        let avg = processor.calculate_average();
        assert!(avg > 0.0);

        let electronics_avg = processor.calculate_category_average("electronics");
        assert_eq!(electronics_avg, 215.25);

        let max_record = processor.find_max_value();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().value(), 250.5);

        let min_record = processor.find_min_value();
        assert!(min_record.is_some());
        assert_eq!(min_record.unwrap().value(), 12.99);
    }
}