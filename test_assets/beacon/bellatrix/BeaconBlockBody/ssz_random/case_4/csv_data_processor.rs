use std::error::Error;
use std::fs::File;
use csv::{Reader, Writer};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        Self {
            id,
            category,
            value,
            active,
        }
    }
}

struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = Reader::from_reader(file);
        
        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }
        
        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .collect()
    }

    fn filter_active(&self) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&DataRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
    }

    fn export_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = Writer::from_writer(file);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    fn clear_records(&mut self) {
        self.records.clear();
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.add_record(DataRecord::new(1, "A".to_string(), 10.5, true));
    processor.add_record(DataRecord::new(2, "B".to_string(), 20.3, false));
    processor.add_record(DataRecord::new(3, "A".to_string(), 15.7, true));
    processor.add_record(DataRecord::new(4, "C".to_string(), 8.9, true));
    
    let category_a_records = processor.filter_by_category("A");
    println!("Category A records: {}", category_a_records.len());
    
    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());
    
    let average_value = processor.calculate_average();
    println!("Average value: {:.2}", average_value);
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record ID: {}", max_record.id);
    }
    
    processor.export_to_csv("output_data.csv")?;
    
    Ok(())
}