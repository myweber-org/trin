
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, category: String, value: f64, active: bool) -> Self {
        DataRecord {
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
        DataProcessor {
            records: Vec::new(),
        }
    }

    fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);

        for result in csv_reader.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    fn filter_active(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .cloned()
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<DataRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .cloned()
    }

    fn save_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new().from_writer(writer);

        csv_writer.write_record(&["id", "category", "value", "active"])?;

        for record in &self.records {
            csv_writer.write_record(&[
                record.id.to_string(),
                record.category.clone(),
                record.value.to_string(),
                record.active.to_string(),
            ])?;
        }

        csv_writer.flush()?;
        Ok(())
    }

    fn add_record(&mut self, record: DataRecord) {
        self.records.push(record);
    }

    fn remove_record_by_id(&mut self, id: u32) -> bool {
        let initial_len = self.records.len();
        self.records.retain(|record| record.id != id);
        self.records.len() < initial_len
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();

    processor.add_record(DataRecord::new(1, "A".to_string(), 100.5, true));
    processor.add_record(DataRecord::new(2, "B".to_string(), 200.3, false));
    processor.add_record(DataRecord::new(3, "A".to_string(), 150.7, true));
    processor.add_record(DataRecord::new(4, "C".to_string(), 75.2, true));

    let category_a_records = processor.filter_by_category("A");
    println!("Category A records: {:?}", category_a_records);

    let active_records = processor.filter_active();
    println!("Active records: {:?}", active_records);

    let average_value = processor.calculate_average();
    println!("Average value: {:.2}", average_value);

    if let Some(max_record) = processor.find_max_value() {
        println!("Record with max value: {:?}", max_record);
    }

    processor.save_to_csv("output_data.csv")?;

    let removed = processor.remove_record_by_id(2);
    println!("Record with id 2 removed: {}", removed);

    Ok(())
}

fn main() {
    if let Err(e) = process_data_sample() {
        eprintln!("Error processing data: {}", e);
    }
}