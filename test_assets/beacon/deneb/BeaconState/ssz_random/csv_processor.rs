use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 4 {
                return Err(format!("Invalid CSV format at line {}", index + 1).into());
            }
            
            let record = CsvRecord {
                id: parts[0].parse()?,
                name: parts[1].to_string(),
                value: parts[2].parse()?,
                active: parts[3].parse().unwrap_or(false),
            };
            
            self.records.push(record);
        }
        
        Ok(self.records.len())
    }

    pub fn filter_by_value(&self, threshold: f64) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.value > threshold && record.active)
            .collect()
    }

    pub fn calculate_average(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        Some(sum / self.records.len() as f64)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&CsvRecord> {
        self.records
            .iter()
            .find(|record| record.name.to_lowercase() == name.to_lowercase())
    }
}

pub fn process_csv_data() -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new();
    let count = processor.load_from_file("data.csv")?;
    
    println!("Loaded {} records", count);
    
    if let Some(avg) = processor.calculate_average() {
        println!("Average value: {:.2}", avg);
    }
    
    let high_value_records = processor.filter_by_value(100.0);
    println!("Records with value > 100: {}", high_value_records.len());
    
    match processor.find_by_name("example") {
        Some(record) => println!("Found record: {:?}", record),
        None => println!("Record not found"),
    }
    
    Ok(())
}