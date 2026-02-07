
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    category: String,
    value: f64,
    active: bool,
}

impl Record {
    fn new(id: u32, name: &str, category: &str, value: f64, active: bool) -> Self {
        Record {
            id,
            name: name.to_string(),
            category: category.to_string(),
            value,
            active,
        }
    }

    fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }

    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value >= 0.0
    }
}

fn load_records_from_csv(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
    
    let mut records = Vec::new();
    
    for result in csv_reader.deserialize() {
        let record: Record = result?;
        if record.is_valid() {
            records.push(record);
        }
    }
    
    Ok(records)
}

fn filter_records_by_category(records: &[Record], category_filter: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category_filter && r.active)
        .cloned()
        .collect()
}

fn process_records(records: &mut [Record], multiplier: f64) {
    for record in records.iter_mut() {
        record.transform_value(multiplier);
    }
}

fn save_records_to_csv(records: &[Record], output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    let mut csv_writer = WriterBuilder::new().from_writer(writer);
    
    for record in records {
        csv_writer.serialize(record)?;
    }
    
    csv_writer.flush()?;
    Ok(())
}

fn calculate_statistics(records: &[Record]) -> (f64, f64, f64) {
    if records.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let count = records.len() as f64;
    let mean = sum / count;
    
    let variance: f64 = records
        .iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/processed_output.csv";
    
    match load_records_from_csv(input_file) {
        Ok(mut records) => {
            println!("Loaded {} valid records", records.len());
            
            let filtered_records = filter_records_by_category(&records, "premium");
            println!("Found {} premium records", filtered_records.len());
            
            let mut processable_records = filtered_records.clone();
            process_records(&mut processable_records, 1.15);
            
            let (total, average, deviation) = calculate_statistics(&processable_records);
            println!("Statistics - Total: {:.2}, Average: {:.2}, Std Dev: {:.2}", 
                     total, average, deviation);
            
            save_records_to_csv(&processable_records, output_file)?;
            println!("Processed data saved to {}", output_file);
            
            Ok(())
        }
        Err(e) => {
            eprintln!("Error loading CSV: {}", e);
            Err(e)
        }
    }
}