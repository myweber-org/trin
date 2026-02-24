use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    active: bool,
}

fn is_valid_record(record: &Record) -> bool {
    !record.name.is_empty() && record.value >= 0.0
}

fn clean_data(input_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = Reader::from_path(input_path)?;
    let mut writer = Writer::from_path(output_path)?;

    for result in reader.deserialize() {
        let record: Record = result?;
        if is_valid_record(&record) {
            writer.serialize(&record)?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "raw_data.csv";
    let output_file = "cleaned_data.csv";
    
    clean_data(input_file, output_file)?;
    println!("Data cleaning completed successfully");
    Ok(())
}
use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_null_values(&mut self) -> &mut Self
    where
        T: Default + PartialEq,
    {
        let default_value = T::default();
        self.data.retain(|item| *item != default_value);
        self
    }

    pub fn remove_duplicates(&mut self) -> &mut Self {
        let mut seen = HashSet::new();
        self.data.retain(|item| seen.insert(item.clone()));
        self
    }

    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_duplicates() {
        let data = vec![1, 2, 2, 3, 4, 4, 5];
        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_duplicates();
        assert_eq!(cleaner.get_data(), &vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_remove_null_values() {
        let data = vec![0, 1, 0, 2, 3, 0];
        let mut cleaner = DataCleaner::new(data);
        cleaner.remove_null_values();
        assert_eq!(cleaner.get_data(), &vec![1, 2, 3]);
    }
}