use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

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
}

struct DataProcessor {
    records: Vec<Record>,
}

impl DataProcessor {
    fn new() -> Self {
        DataProcessor { records: Vec::new() }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }
        
        println!("Loaded {} records from {}", self.records.len(), file_path);
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category && r.active)
            .cloned()
            .collect()
    }

    fn calculate_total_value(&self) -> f64 {
        self.records
            .iter()
            .filter(|r| r.active)
            .map(|r| r.value)
            .sum()
    }

    fn calculate_average_value(&self) -> f64 {
        let active_records: Vec<&Record> = self.records
            .iter()
            .filter(|r| r.active)
            .collect();
        
        if active_records.is_empty() {
            return 0.0;
        }
        
        let total: f64 = active_records.iter().map(|r| r.value).sum();
        total / active_records.len() as f64
    }

    fn export_to_csv(&self, file_path: &str, records: &[Record]) -> Result<(), Box<dyn Error>> {
        let mut wtr = Writer::from_path(file_path)?;
        
        for record in records {
            wtr.serialize(record)?;
        }
        
        wtr.flush()?;
        println!("Exported {} records to {}", records.len(), file_path);
        Ok(())
    }

    fn find_max_value_record(&self) -> Option<&Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    fn find_min_value_record(&self) -> Option<&Record> {
        self.records
            .iter()
            .filter(|r| r.active)
            .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }
}

fn generate_sample_data() -> Vec<Record> {
    vec![
        Record::new(1, "Item A", "Electronics", 299.99, true),
        Record::new(2, "Item B", "Books", 24.99, true),
        Record::new(3, "Item C", "Electronics", 499.99, false),
        Record::new(4, "Item D", "Clothing", 59.99, true),
        Record::new(5, "Item E", "Books", 14.99, true),
        Record::new(6, "Item F", "Electronics", 199.99, true),
        Record::new(7, "Item G", "Clothing", 79.99, true),
        Record::new(8, "Item H", "Books", 34.99, false),
    ]
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.records = generate_sample_data();
    
    println!("Total active value: ${:.2}", processor.calculate_total_value());
    println!("Average active value: ${:.2}", processor.calculate_average_value());
    
    if let Some(max_record) = processor.find_max_value_record() {
        println!("Highest value item: {} (${})", max_record.name, max_record.value);
    }
    
    if let Some(min_record) = processor.find_min_value_record() {
        println!("Lowest value item: {} (${})", min_record.name, min_record.value);
    }
    
    let electronics = processor.filter_by_category("Electronics");
    println!("Active Electronics items: {}", electronics.len());
    
    for item in &electronics {
        println!("  - {}: ${}", item.name, item.value);
    }
    
    processor.export_to_csv("active_electronics.csv", &electronics)?;
    
    Ok(())
}