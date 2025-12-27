use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    id: u32,
    name: String,
    age: u8,
    active: bool,
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn filter_by_age(&self, min_age: u8, max_age: u8) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.age >= min_age && record.age <= max_age)
            .collect()
    }

    fn save_to_csv(&self, file_path: &str, records: Vec<&Record>) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);

        for record in records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn calculate_average_age(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let total: u32 = self.records.iter().map(|r| r.age as u32).sum();
        total as f64 / self.records.len() as f64
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input.csv")?;
    
    println!("Total records loaded: {}", processor.records.len());
    println!("Average age: {:.2}", processor.calculate_average_age());
    
    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());
    
    processor.save_to_csv("active_records.csv", active_records)?;
    
    let age_filtered = processor.filter_by_age(25, 40);
    println!("Records between 25-40: {}", age_filtered.len());
    
    processor.save_to_csv("age_filtered.csv", age_filtered)?;
    
    Ok(())
}