
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 5 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    name: parts[1].to_string(),
                    category: parts[2].to_string(),
                    value: parts[3].parse()?,
                    active: parts[4].parse().unwrap_or(false),
                };
                self.records.push(record);
            }
        }
        
        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_value() / self.records.len() as f64
    }

    pub fn find_max_value_record(&self) -> Option<CsvRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .cloned()
    }

    pub fn find_min_value_record(&self) -> Option<CsvRecord> {
        self.records
            .iter()
            .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .cloned()
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.records
            .iter()
            .map(|record| record.category.clone())
            .collect();
        
        categories.sort();
        categories.dedup();
        categories
    }

    pub fn aggregate_by_category(&self) -> Vec<(String, f64)> {
        let mut aggregates = std::collections::HashMap::new();
        
        for record in &self.records {
            *aggregates.entry(record.category.clone()).or_insert(0.0) += record.value;
        }
        
        let mut result: Vec<(String, f64)> = aggregates.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_processor() {
        let processor = CsvProcessor::new();
        assert_eq!(processor.count_records(), 0);
        assert_eq!(processor.calculate_total_value(), 0.0);
        assert_eq!(processor.calculate_average_value(), 0.0);
    }

    #[test]
    fn test_record_creation() {
        let record = CsvRecord {
            id: 1,
            name: "Test Item".to_string(),
            category: "Electronics".to_string(),
            value: 99.99,
            active: true,
        };
        
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "Test Item");
        assert_eq!(record.category, "Electronics");
        assert_eq!(record.value, 99.99);
        assert_eq!(record.active, true);
    }
}