use std::collections::HashSet;

pub struct DataCleaner {
    data: Vec<Vec<Option<String>>>,
}

impl DataCleaner {
    pub fn new(data: Vec<Vec<Option<String>>>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_rows(&mut self) {
        self.data.retain(|row| {
            row.iter().all(|cell| cell.is_some())
        });
    }

    pub fn deduplicate(&mut self) {
        let mut seen = HashSet::new();
        self.data.retain(|row| {
            let key: Vec<&str> = row.iter()
                .map(|cell| cell.as_deref().unwrap_or(""))
                .collect();
            seen.insert(key)
        });
    }

    pub fn get_data(&self) -> &Vec<Vec<Option<String>>> {
        &self.data
    }

    pub fn clean(&mut self) {
        self.remove_null_rows();
        self.deduplicate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleaner_removes_nulls() {
        let data = vec![
            vec![Some("A".to_string()), Some("B".to_string())],
            vec![Some("C".to_string()), None],
            vec![Some("E".to_string()), Some("F".to_string())],
        ];
        
        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_rows();
        
        assert_eq!(cleaner.get_data().len(), 2);
    }

    #[test]
    fn test_cleaner_deduplicates() {
        let data = vec![
            vec![Some("X".to_string()), Some("Y".to_string())],
            vec![Some("X".to_string()), Some("Y".to_string())],
            vec![Some("Z".to_string()), Some("W".to_string())],
        ];
        
        let mut cleaner = DataCleaner::new(data);
        cleaner.deduplicate();
        
        assert_eq!(cleaner.get_data().len(), 2);
    }
}
use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

fn normalize_string(s: &str) -> String {
    s.trim().to_lowercase()
}

fn filter_record(record: &Record) -> bool {
    record.value >= 0.0 && !record.name.is_empty()
}

fn process_csv(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let mut record: Record = result?;
        
        record.name = normalize_string(&record.name);
        record.category = normalize_string(&record.category);
        
        if filter_record(&record) {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";
    
    process_csv(input_file, output_file)?;
    
    println!("Data cleaning completed successfully");
    Ok(())
}