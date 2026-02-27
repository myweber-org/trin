
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    pub fn new(id: u32, name: String, category: String, value: f64, active: bool) -> Self {
        Record {
            id,
            name,
            category,
            value,
            active,
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

pub struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    pub fn from_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            if index == 0 {
                continue;
            }

            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();
            
            if parts.len() == 5 {
                let id = parts[0].parse::<u32>()?;
                let name = parts[1].to_string();
                let category = parts[2].to_string();
                let value = parts[3].parse::<f64>()?;
                let active = parts[4].parse::<bool>().unwrap_or(false);

                records.push(Record::new(id, name, category, value, active));
            }
        }

        Ok(DataProcessor { records })
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    pub fn filter_active(&self) -> Vec<&Record> {
        self.records
            .iter()
            .filter(|record| record.is_active())
            .collect()
    }

    pub fn calculate_total(&self) -> f64 {
        self.records.iter().map(|record| record.value()).sum()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total() / self.records.len() as f64
    }

    pub fn group_by_category(&self) -> Vec<(String, f64)> {
        let mut categories = std::collections::HashMap::new();
        
        for record in &self.records {
            let entry = categories.entry(record.category.clone()).or_insert(0.0);
            *entry += record.value();
        }

        categories.into_iter().collect()
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value().partial_cmp(&b.value()).unwrap()
        })
    }

    pub fn find_min_value(&self) -> Option<&Record> {
        self.records.iter().min_by(|a, b| {
            a.value().partial_cmp(&b.value()).unwrap()
        })
    }

    pub fn count_records(&self) -> usize {
        self.records.len()
    }
}

pub fn process_data_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    let processor = DataProcessor::from_csv(file_path)?;
    
    println!("Total records: {}", processor.count_records());
    println!("Total value: {:.2}", processor.calculate_total());
    println!("Average value: {:.2}", processor.calculate_average());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Maximum value record: {:?}", max_record);
    }
    
    if let Some(min_record) = processor.find_min_value() {
        println!("Minimum value record: {:?}", min_record);
    }
    
    let categories = processor.group_by_category();
    for (category, total) in categories {
        println!("Category '{}' total: {:.2}", category, total);
    }
    
    Ok(())
}