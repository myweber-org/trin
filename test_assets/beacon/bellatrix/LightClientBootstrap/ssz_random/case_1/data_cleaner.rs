use csv::{ReaderBuilder, WriterBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

struct DataCleaner {
    input_path: String,
    output_path: String,
    min_age: u8,
}

impl DataCleaner {
    fn new(input_path: &str, output_path: &str, min_age: u8) -> Self {
        DataCleaner {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            min_age,
        }
    }

    fn clean_data(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(input_file);

        let output_file = File::create(&self.output_path)?;
        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .from_writer(output_file);

        let mut processed_count = 0;

        for result in rdr.deserialize() {
            let record: Record = result?;
            
            if record.age >= self.min_age && record.active {
                wtr.serialize(&record)?;
                processed_count += 1;
            }
        }

        wtr.flush()?;
        Ok(processed_count)
    }

    fn validate_paths(&self) -> Result<(), Box<dyn Error>> {
        if !Path::new(&self.input_path).exists() {
            return Err("Input file does not exist".into());
        }

        let output_dir = Path::new(&self.output_path)
            .parent()
            .ok_or("Invalid output path")?;

        if !output_dir.exists() {
            return Err("Output directory does not exist".into());
        }

        Ok(())
    }
}

fn process_dataset() -> Result<(), Box<dyn Error>> {
    let cleaner = DataCleaner::new("input.csv", "output/cleaned.csv", 18);
    
    cleaner.validate_paths()?;
    
    let processed = cleaner.clean_data()?;
    println!("Processed {} valid records", processed);
    
    Ok(())
}

fn main() {
    if let Err(e) = process_dataset() {
        eprintln!("Error processing data: {}", e);
        std::process::exit(1);
    }
}
use std::collections::HashMap;

pub struct DataCleaner {
    data: Vec<f64>,
    threshold: f64,
}

impl DataCleaner {
    pub fn new(data: Vec<f64>, threshold: f64) -> Self {
        DataCleaner { data, threshold }
    }

    pub fn remove_outliers(&mut self) -> Vec<f64> {
        if self.data.len() < 4 {
            return self.data.clone();
        }

        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_index = (sorted_data.len() as f64 * 0.25).floor() as usize;
        let q3_index = (sorted_data.len() as f64 * 0.75).floor() as usize;

        let q1 = sorted_data[q1_index];
        let q3 = sorted_data[q3_index];
        let iqr = q3 - q1;

        let lower_bound = q1 - self.threshold * iqr;
        let upper_bound = q3 + self.threshold * iqr;

        self.data
            .iter()
            .filter(|&&x| x >= lower_bound && x <= upper_bound)
            .cloned()
            .collect()
    }

    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        if self.data.is_empty() {
            return stats;
        }

        let sum: f64 = self.data.iter().sum();
        let mean = sum / self.data.len() as f64;
        
        let variance: f64 = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.data.len() as f64;
        
        let std_dev = variance.sqrt();

        stats.insert("mean".to_string(), mean);
        stats.insert("std_dev".to_string(), std_dev);
        stats.insert("count".to_string(), self.data.len() as f64);
        stats.insert("sum".to_string(), sum);

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_outliers() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0];
        let mut cleaner = DataCleaner::new(data, 1.5);
        let cleaned = cleaner.remove_outliers();
        
        assert_eq!(cleaned.len(), 5);
        assert!(!cleaned.contains(&100.0));
    }

    #[test]
    fn test_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cleaner = DataCleaner::new(data, 1.5);
        let stats = cleaner.get_statistics();
        
        assert_eq!(stats.get("mean").unwrap(), &3.0);
        assert_eq!(stats.get("count").unwrap(), &5.0);
    }
}
use std::collections::HashSet;

pub struct DataCleaner {
    unique_items: HashSet<String>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            unique_items: HashSet::new(),
        }
    }

    pub fn process_string(&mut self, input: &str) -> Option<String> {
        let normalized = input.trim().to_lowercase();
        
        if normalized.is_empty() {
            return None;
        }

        if self.unique_items.contains(&normalized) {
            return None;
        }

        self.unique_items.insert(normalized.clone());
        Some(normalized)
    }

    pub fn process_batch(&mut self, inputs: &[&str]) -> Vec<String> {
        inputs
            .iter()
            .filter_map(|&input| self.process_string(input))
            .collect()
    }

    pub fn get_unique_count(&self) -> usize {
        self.unique_items.len()
    }

    pub fn clear(&mut self) {
        self.unique_items.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cleaning() {
        let mut cleaner = DataCleaner::new();
        
        assert_eq!(cleaner.process_string("  HELLO  "), Some("hello".to_string()));
        assert_eq!(cleaner.process_string("hello"), None);
        assert_eq!(cleaner.process_string(""), None);
        assert_eq!(cleaner.get_unique_count(), 1);
    }

    #[test]
    fn test_batch_processing() {
        let mut cleaner = DataCleaner::new();
        let inputs = vec!["Apple", "apple", "BANANA", "  banana  ", "Cherry"];
        
        let results = cleaner.process_batch(&inputs);
        assert_eq!(results.len(), 3);
        assert_eq!(cleaner.get_unique_count(), 3);
    }
}