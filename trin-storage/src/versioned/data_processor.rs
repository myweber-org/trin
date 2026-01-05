
use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRecord {
    id: u32,
    name: String,
    value: f64,
    tags: Vec<String>,
    metadata: HashMap<String, String>,
}

impl DataRecord {
    pub fn new(id: u32, name: String, value: f64) -> Self {
        Self {
            id,
            name,
            value,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.name.is_empty() {
            return Err("Name cannot be empty".into());
        }
        if self.value < 0.0 {
            return Err("Value must be non-negative".into());
        }
        if self.id == 0 {
            return Err("ID must be greater than zero".into());
        }
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn transform_value<F>(&mut self, transformer: F)
    where
        F: Fn(f64) -> f64,
    {
        self.value = transformer(self.value);
    }
}

pub fn process_records(records: &mut [DataRecord]) -> Result<Vec<DataRecord>, Box<dyn Error>> {
    let mut processed = Vec::new();
    
    for record in records {
        record.validate()?;
        
        let mut processed_record = DataRecord::new(
            record.id,
            record.name.to_uppercase(),
            record.value * 1.1,
        );
        
        for tag in &record.tags {
            processed_record.add_tag(tag.clone());
        }
        
        for (key, value) in &record.metadata {
            processed_record.set_metadata(key.clone(), value.clone());
        }
        
        processed.push(processed_record);
    }
    
    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_validation() {
        let valid_record = DataRecord::new(1, "Test".to_string(), 100.0);
        assert!(valid_record.validate().is_ok());

        let invalid_record = DataRecord::new(0, "".to_string(), -10.0);
        assert!(invalid_record.validate().is_err());
    }

    #[test]
    fn test_value_transformation() {
        let mut record = DataRecord::new(1, "Test".to_string(), 50.0);
        record.transform_value(|x| x * 2.0);
        assert_eq!(record.value, 100.0);
    }

    #[test]
    fn test_record_processing() {
        let mut records = vec![
            DataRecord::new(1, "alpha".to_string(), 10.0),
            DataRecord::new(2, "beta".to_string(), 20.0),
        ];
        
        let result = process_records(&mut records);
        assert!(result.is_ok());
        
        let processed = result.unwrap();
        assert_eq!(processed[0].name, "ALPHA");
        assert_eq!(processed[1].value, 22.0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            value,
            active,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
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

            let id = parts[0].parse::<u32>().unwrap_or(0);
            let name = parts[1].to_string();
            let value = parts[2].parse::<f64>().unwrap_or(0.0);
            let active = parts[3].parse::<bool>().unwrap_or(false);

            let record = Record::new(id, name, value, active);
            if record.is_valid() {
                self.records.push(record);
                count += 1;
            }
        }

        Ok(count)
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Record> {
        self.records.iter().find(|record| record.name == name)
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_record_validation() {
        let valid_record = Record::new(1, "test".to_string(), 10.5, true);
        assert!(valid_record.is_valid());

        let invalid_record = Record::new(2, "".to_string(), -5.0, false);
        assert!(!invalid_record.is_valid());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert_eq!(processor.get_record_count(), 0);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,item1,10.5,true").unwrap();
        writeln!(temp_file, "2,item2,20.0,false").unwrap();
        writeln!(temp_file, "3,item3,15.75,true").unwrap();

        let count = processor.load_from_csv(temp_file.path()).unwrap();
        assert_eq!(count, 3);
        assert_eq!(processor.get_record_count(), 3);
        assert_eq!(processor.filter_active().len(), 2);
        assert_eq!(processor.calculate_total(), 46.25);

        let found = processor.find_by_name("item2");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 2);

        processor.clear();
        assert_eq!(processor.get_record_count(), 0);
    }
}
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct DataProcessor {
    data: Vec<f64>,
    frequency_map: HashMap<String, u32>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            data: Vec::new(),
            frequency_map: HashMap::new(),
        }
    }

    pub fn load_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            for part in parts {
                if let Ok(value) = part.trim().parse::<f64>() {
                    self.data.push(value);
                } else {
                    self.frequency_map
                        .entry(part.trim().to_string())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
            }
        }
        
        Ok(())
    }

    pub fn calculate_statistics(&self) -> (f64, f64, f64, f64) {
        if self.data.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let sum: f64 = self.data.iter().sum();
        let count = self.data.len() as f64;
        let mean = sum / count;

        let variance: f64 = self.data
            .iter()
            .map(|value| {
                let diff = mean - *value;
                diff * diff
            })
            .sum::<f64>() / count;

        let std_dev = variance.sqrt();

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let median = if count as usize % 2 == 0 {
            let mid = count as usize / 2;
            (sorted_data[mid - 1] + sorted_data[mid]) / 2.0
        } else {
            sorted_data[count as usize / 2]
        };

        (mean, median, variance, std_dev)
    }

    pub fn get_top_categories(&self, limit: usize) -> Vec<(String, u32)> {
        let mut entries: Vec<_> = self.frequency_map.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        
        entries
            .iter()
            .take(limit)
            .map(|(key, value)| (key.clone(), *value))
            .collect()
    }

    pub fn filter_data(&self, predicate: impl Fn(f64) -> bool) -> Vec<f64> {
        self.data
            .iter()
            .filter(|&&value| predicate(value))
            .copied()
            .collect()
    }

    pub fn normalize_data(&mut self) {
        let (mean, _, _, std_dev) = self.calculate_statistics();
        
        if std_dev > 0.0 {
            for value in &mut self.data {
                *value = (*value - mean) / std_dev;
            }
        }
    }

    pub fn export_summary(&self) -> String {
        let (mean, median, variance, std_dev) = self.calculate_statistics();
        let top_categories = self.get_top_categories(5);
        
        let mut summary = format!(
            "Statistical Summary:\n\
             Mean: {:.4}\n\
             Median: {:.4}\n\
             Variance: {:.4}\n\
             Standard Deviation: {:.4}\n\
             Data Points: {}\n\
             Unique Categories: {}\n\n\
             Top Categories:\n",
            mean, median, variance, std_dev,
            self.data.len(),
            self.frequency_map.len()
        );

        for (category, count) in top_categories {
            summary.push_str(&format!("{}: {}\n", category, count));
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_processing() {
        let mut processor = DataProcessor::new();
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "10.5,20.3,15.7,CategoryA,CategoryB").unwrap();
        writeln!(temp_file, "8.2,12.9,CategoryA,25.1,CategoryC").unwrap();
        
        let result = processor.load_csv(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let stats = processor.calculate_statistics();
        assert!(stats.0 > 0.0);
        
        let top_categories = processor.get_top_categories(2);
        assert_eq!(top_categories.len(), 2);
        
        processor.normalize_data();
        let normalized_stats = processor.calculate_statistics();
        assert!(normalized_stats.0.abs() < 0.0001);
    }
}