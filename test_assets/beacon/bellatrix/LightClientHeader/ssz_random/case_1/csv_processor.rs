use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct Record {
    id: u32,
    name: String,
    value: f64,
    category: String,
}

impl Record {
    fn new(id: u32, name: String, value: f64, category: String) -> Self {
        Record {
            id,
            name,
            value,
            category,
        }
    }

    fn transform_value(&mut self, multiplier: f64) {
        self.value *= multiplier;
    }
}

fn load_records(file_path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
    
    let mut records = Vec::new();
    for result in csv_reader.deserialize() {
        let record: Record = result?;
        records.push(record);
    }
    
    Ok(records)
}

fn filter_records(records: &[Record], category_filter: &str) -> Vec<Record> {
    records
        .iter()
        .filter(|r| r.category == category_filter)
        .cloned()
        .collect()
}

fn process_records(records: &mut [Record], multiplier: f64) {
    for record in records.iter_mut() {
        record.transform_value(multiplier);
    }
}

fn save_records(records: &[Record], output_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    let mut csv_writer = WriterBuilder::new().from_writer(writer);
    
    for record in records {
        csv_writer.serialize(record)?;
    }
    
    csv_writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "data/input.csv";
    let output_file = "data/processed.csv";
    
    let mut records = load_records(input_file)?;
    println!("Loaded {} records", records.len());
    
    let filtered_records = filter_records(&records, "premium");
    println!("Filtered to {} premium records", filtered_records.len());
    
    let mut records_to_process = filtered_records.clone();
    process_records(&mut records_to_process, 1.15);
    
    save_records(&records_to_process, output_file)?;
    println!("Processed records saved to {}", output_file);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_creation() {
        let record = Record::new(1, "test".to_string(), 100.0, "standard".to_string());
        assert_eq!(record.id, 1);
        assert_eq!(record.name, "test");
        assert_eq!(record.value, 100.0);
        assert_eq!(record.category, "standard");
    }

    #[test]
    fn test_value_transformation() {
        let mut record = Record::new(1, "test".to_string(), 100.0, "standard".to_string());
        record.transform_value(1.5);
        assert_eq!(record.value, 150.0);
    }

    #[test]
    fn test_filtering() {
        let records = vec![
            Record::new(1, "a".to_string(), 10.0, "premium".to_string()),
            Record::new(2, "b".to_string(), 20.0, "standard".to_string()),
            Record::new(3, "c".to_string(), 30.0, "premium".to_string()),
        ];
        
        let filtered = filter_records(&records, "premium");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "premium"));
    }
}