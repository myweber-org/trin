use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

#[derive(Debug)]
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
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    fn filter_active(&self) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .cloned()
            .collect()
    }

    fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }

    fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn save_to_csv(&self, file_path: &str, records: &[Record]) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        for record in records {
            csv_writer.serialize(record)?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    fn aggregate_by_category(&self) -> Vec<(String, f64)> {
        let mut aggregates = std::collections::HashMap::new();

        for record in &self.records {
            let entry = aggregates.entry(record.category.clone()).or_insert(0.0);
            *entry += record.value;
        }

        aggregates.into_iter().collect()
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input_data.csv")?;
    
    let active_records = processor.filter_active();
    println!("Active records count: {}", active_records.len());
    
    let tech_records = processor.filter_by_category("Technology");
    println!("Technology records count: {}", tech_records.len());
    
    let total_value = processor.calculate_total_value();
    println!("Total value: {:.2}", total_value);
    
    let avg_value = processor.calculate_average_value();
    println!("Average value: {:.2}", avg_value);
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: {:?}", max_record);
    }
    
    let aggregates = processor.aggregate_by_category();
    println!("Aggregates by category: {:?}", aggregates);
    
    processor.save_to_csv("filtered_data.csv", &active_records)?;
    
    Ok(())
}

fn main() {
    if let Err(e) = process_data_sample() {
        eprintln!("Error processing data: {}", e);
        std::process::exit(1);
    }
}