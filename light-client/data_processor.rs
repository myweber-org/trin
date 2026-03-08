
use csv::Reader;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize)]
struct Record {
    id: u32,
    category: String,
    value: f64,
    timestamp: String,
}

pub struct DataProcessor {
    records: Vec<Record>,
    statistics: HashMap<String, f64>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
            statistics: HashMap::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        println!("Loaded {} records from {}", self.records.len(), file_path);
        Ok(())
    }

    pub fn calculate_statistics(&mut self) {
        let mut category_sums: HashMap<String, f64> = HashMap::new();
        let mut category_counts: HashMap<String, u32> = HashMap::new();
        
        for record in &self.records {
            let sum = category_sums.entry(record.category.clone()).or_insert(0.0);
            *sum += record.value;
            
            let count = category_counts.entry(record.category.clone()).or_insert(0);
            *count += 1;
        }
        
        for (category, sum) in category_sums {
            if let Some(&count) = category_counts.get(&category) {
                if count > 0 {
                    self.statistics.insert(category, sum / count as f64);
                }
            }
        }
    }

    pub fn get_average(&self, category: &str) -> Option<f64> {
        self.statistics.get(category).copied()
    }

    pub fn filter_by_threshold(&self, threshold: f64) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.value > threshold)
            .collect()
    }

    pub fn export_summary(&self) -> String {
        let mut summary = String::from("Data Processing Summary\n");
        summary.push_str(&format!("Total Records: {}\n", self.records.len()));
        summary.push_str("Category Averages:\n");
        
        for (category, avg) in &self.statistics {
            summary.push_str(&format!("  {}: {:.2}\n", category, avg));
        }
        
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,category,value,timestamp").unwrap();
        writeln!(temp_file, "1,electronics,150.5,2024-01-15").unwrap();
        writeln!(temp_file, "2,furniture,89.99,2024-01-16").unwrap();
        writeln!(temp_file, "3,electronics,200.0,2024-01-17").unwrap();
        
        let result = processor.load_from_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 3);
        
        processor.calculate_statistics();
        
        let electronics_avg = processor.get_average("electronics");
        assert!(electronics_avg.is_some());
        assert!((electronics_avg.unwrap() - 175.25).abs() < 0.01);
        
        let high_value_records = processor.filter_by_threshold(100.0);
        assert_eq!(high_value_records.len(), 2);
        
        let summary = processor.export_summary();
        assert!(summary.contains("Total Records: 3"));
        assert!(summary.contains("electronics"));
    }
}