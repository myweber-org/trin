use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub timestamp: String,
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
                    timestamp: parts[4].to_string(),
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

    pub fn aggregate_by_category(&self) -> HashMap<String, f64> {
        let mut aggregates = HashMap::new();
        
        for record in &self.records {
            let entry = aggregates.entry(record.category.clone()).or_insert(0.0);
            *entry += record.value;
        }
        
        aggregates
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    pub fn get_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let total: f64 = self.records.iter().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_records_sorted_by_value(&self) -> Vec<CsvRecord> {
        let mut sorted = self.records.clone();
        sorted.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,category,value,timestamp").unwrap();
        writeln!(file, "1,ItemA,Electronics,250.50,2023-01-15").unwrap();
        writeln!(file, "2,ItemB,Furniture,150.75,2023-02-20").unwrap();
        writeln!(file, "3,ItemC,Electronics,300.25,2023-03-10").unwrap();
        writeln!(file, "4,ItemD,Clothing,75.30,2023-04-05").unwrap();
        file
    }

    #[test]
    fn test_load_and_filter() {
        let test_file = create_test_csv();
        let mut processor = CsvProcessor::new();
        
        processor.load_from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let aggregates = processor.aggregate_by_category();
        assert_eq!(aggregates.get("Electronics"), Some(&550.75));
        
        assert_eq!(processor.count_records(), 4);
        assert!((processor.get_average_value() - 194.2).abs() < 0.1);
    }
}