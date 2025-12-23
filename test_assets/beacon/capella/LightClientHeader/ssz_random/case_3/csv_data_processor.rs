use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
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

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let record = CsvRecord {
                    id: parts[0].parse()?,
                    category: parts[1].to_string(),
                    value: parts[2].parse()?,
                    timestamp: parts[3].to_string(),
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

    pub fn calculate_average_by_category(&self) -> HashMap<String, f64> {
        let mut category_sums: HashMap<String, (f64, usize)> = HashMap::new();
        
        for record in &self.records {
            let entry = category_sums
                .entry(record.category.clone())
                .or_insert((0.0, 0));
            entry.0 += record.value;
            entry.1 += 1;
        }
        
        category_sums
            .into_iter()
            .map(|(category, (sum, count))| (category, sum / count as f64))
            .collect()
    }

    pub fn get_total_records(&self) -> usize {
        self.records.len()
    }

    pub fn get_max_value(&self) -> Option<f64> {
        self.records
            .iter()
            .map(|record| record.value)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    pub fn get_min_value(&self) -> Option<f64> {
        self.records
            .iter()
            .map(|record| record.value)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
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
                        1,electronics,299.99,2023-10-01\n\
                        2,books,19.99,2023-10-02\n\
                        3,electronics,599.99,2023-10-03\n\
                        4,books,29.99,2023-10-04";
        
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", csv_data).unwrap();
        
        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_total_records(), 4);
        
        let electronics = processor.filter_by_category("electronics");
        assert_eq!(electronics.len(), 2);
        
        let averages = processor.calculate_average_by_category();
        assert_eq!(averages.get("electronics").unwrap(), &449.99);
        assert_eq!(averages.get("books").unwrap(), &24.99);
        
        assert_eq!(processor.get_max_value(), Some(599.99));
        assert_eq!(processor.get_min_value(), Some(19.99));
    }
}