
use std::collections::HashMap;

pub struct DataCleaner {
    filters: Vec<Box<dyn Fn(&HashMap<String, String>) -> bool>>,
}

impl DataCleaner {
    pub fn new() -> Self {
        DataCleaner {
            filters: Vec::new(),
        }
    }

    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&HashMap<String, String>) -> bool + 'static,
    {
        self.filters.push(Box::new(filter));
    }

    pub fn clean(&self, data: Vec<HashMap<String, String>>) -> Vec<HashMap<String, String>> {
        data.into_iter()
            .filter(|entry| self.filters.iter().all(|filter| filter(entry)))
            .collect()
    }
}

pub fn create_default_cleaner() -> DataCleaner {
    let mut cleaner = DataCleaner::new();
    
    cleaner.add_filter(|entry| {
        entry.contains_key("id") && !entry.get("id").unwrap().is_empty()
    });
    
    cleaner.add_filter(|entry| {
        entry.get("timestamp")
            .and_then(|ts| ts.parse::<u64>().ok())
            .map_or(false, |timestamp| timestamp > 0)
    });
    
    cleaner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_cleaner() {
        let cleaner = create_default_cleaner();
        
        let mut valid_entry = HashMap::new();
        valid_entry.insert("id".to_string(), "123".to_string());
        valid_entry.insert("timestamp".to_string(), "1672531200".to_string());
        
        let mut invalid_entry = HashMap::new();
        invalid_entry.insert("id".to_string(), "".to_string());
        invalid_entry.insert("timestamp".to_string(), "0".to_string());
        
        let data = vec![valid_entry.clone(), invalid_entry];
        let cleaned = cleaner.clean(data);
        
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].get("id").unwrap(), "123");
    }
}use csv::{Reader, Writer};
use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Clone)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn clean_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    let mut seen_ids = HashSet::new();
    let mut cleaned_records = Vec::new();

    for result in reader.deserialize() {
        let record: Record = result?;

        if !seen_ids.contains(&record.id) {
            seen_ids.insert(record.id);
            cleaned_records.push(record);
        }
    }

    cleaned_records.sort_by(|a, b| a.id.cmp(&b.id));

    writer.write_record(&["id", "name", "value", "category"])?;
    for record in cleaned_records {
        writer.serialize(record)?;
    }

    writer.flush()?;
    println!("Cleaned {} records to {}", cleaned_records.len(), output_path);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    clean_csv("input_data.csv", "cleaned_data.csv")?;
    Ok(())
}