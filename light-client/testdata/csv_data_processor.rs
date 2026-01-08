use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
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

    pub fn load_from_file(&mut self, filepath: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        
        let mut count = 0;
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 4 {
                continue;
            }
            
            let record = CsvRecord {
                id: parts[0].parse().unwrap_or(0),
                category: parts[1].to_string(),
                value: parts[2].parse().unwrap_or(0.0),
                timestamp: parts[3].to_string(),
            };
            
            self.records.push(record);
            count += 1;
        }
        
        Ok(count)
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn group_by_category(&self) -> HashMap<String, Vec<&CsvRecord>> {
        let mut groups: HashMap<String, Vec<&CsvRecord>> = HashMap::new();
        
        for record in &self.records {
            groups
                .entry(record.category.clone())
                .or_insert_with(Vec::new)
                .push(record);
        }
        
        groups
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    pub fn get_summary(&self) -> String {
        let avg = self.calculate_average();
        let max_record = self.find_max_value();
        let categories = self.group_by_category();
        
        let max_info = match max_record {
            Some(record) => format!("Max value: {} (ID: {})", record.value, record.id),
            None => "No records found".to_string(),
        };
        
        format!(
            "Total records: {}\nAverage value: {:.2}\nUnique categories: {}\n{}",
            self.records.len(),
            avg,
            categories.len(),
            max_info
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let csv_data = "id,category,value,timestamp\n\
                        1,electronics,100.5,2023-01-01\n\
                        2,furniture,250.0,2023-01-02\n\
                        3,electronics,75.3,2023-01-03\n\
                        4,clothing,45.8,2023-01-04";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let mut processor = CsvProcessor::new();
        let count = processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(count, 4);
        assert_eq!(processor.records.len(), 4);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let avg = processor.calculate_average();
        assert!(avg > 0.0);
        
        let groups = processor.group_by_category();
        assert_eq!(groups.len(), 3);
        
        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.value, 250.0);
    }
}