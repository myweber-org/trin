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
        let reader = BufReader::new(file);
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            if index == 0 {
                continue;
            }
            
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 4 {
                let id = parts[0].parse::<u32>()?;
                let category = parts[1].to_string();
                let value = parts[2].parse::<f64>()?;
                let active = parts[3].parse::<bool>().unwrap_or(false);
                
                self.records.push(Record {
                    id,
                    category,
                    value,
                    active,
                });
            }
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn aggregate_by_category(&self) -> HashMap<String, f64> {
        let mut aggregates = HashMap::new();
        
        for record in &self.records {
            let entry = aggregates.entry(record.category.clone()).or_insert(0.0);
            *entry += record.value;
        }
        
        aggregates
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let total: f64 = self.records.iter().map(|r| r.value).sum();
        total / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    fn find_min_value(&self) -> Option<&Record> {
        self.records.iter().min_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap()
        })
    }

    fn get_statistics(&self) -> (f64, Option<&Record>, Option<&Record>) {
        let average = self.calculate_average();
        let max_record = self.find_max_value();
        let min_record = self.find_min_value();
        
        (average, max_record, min_record)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    match processor.load_from_csv("data.csv") {
        Ok(_) => {
            println!("Loaded {} records", processor.records.len());
            
            let electronics = processor.filter_by_category("Electronics");
            println!("Found {} electronics records", electronics.len());
            
            let active_records = processor.filter_active();
            println!("Found {} active records", active_records.len());
            
            let aggregates = processor.aggregate_by_category();
            for (category, total) in aggregates {
                println!("Category {}: total value = {:.2}", category, total);
            }
            
            let (average, max_record, min_record) = processor.get_statistics();
            println!("Average value: {:.2}", average);
            
            if let Some(max) = max_record {
                println!("Max value record: ID {} with value {:.2}", max.id, max.value);
            }
            
            if let Some(min) = min_record {
                println!("Min value record: ID {} with value {:.2}", min.id, min.value);
            }
        }
        Err(e) => {
            eprintln!("Error loading CSV: {}", e);
        }
    }
    
    Ok(())
}