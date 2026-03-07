use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Record {
            id,
            category,
            value,
            active,
        }
    }

    fn is_valid(&self) -> bool {
        self.value >= 0.0 && !self.category.is_empty()
    }
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    fn load_from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 4 {
                let id = parts[0].parse::<u32>().unwrap_or(0);
                let category = parts[1].to_string();
                let value = parts[2].parse::<f64>().unwrap_or(0.0);
                let active = parts[3].parse::<bool>().unwrap_or(false);
                
                self.records.push(Record::new(id, category, value, active));
            }
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category && record.is_valid())
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        let valid_records: Vec<&Record> = self.records
            .iter()
            .filter(|record| record.is_valid())
            .collect();
        
        if valid_records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = valid_records.iter().map(|record| record.value).sum();
        sum / valid_records.len() as f64
    }

    fn group_by_category(&self) -> HashMap<String, Vec<&Record>> {
        let mut groups: HashMap<String, Vec<&Record>> = HashMap::new();
        
        for record in &self.records {
            if record.is_valid() {
                groups
                    .entry(record.category.clone())
                    .or_insert_with(Vec::new)
                    .push(record);
            }
        }
        
        groups
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records
            .iter()
            .filter(|record| record.is_valid())
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_file("data.csv")?;
    
    println!("Total records loaded: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Maximum value record: ID={}, Category={}, Value={}", 
                 max_record.id, max_record.category, max_record.value);
    }
    
    let electronics = processor.filter_by_category("Electronics");
    println!("Electronics records: {}", electronics.len());
    
    let groups = processor.group_by_category();
    for (category, records) in groups {
        println!("Category '{}' has {} valid records", category, records.len());
    }
    
    Ok(())
}

fn main() {
    if let Err(e) = process_data_sample() {
        eprintln!("Processing error: {}", e);
    }
}