use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    timestamp: String,
}

impl DataRecord {
    fn new(id: u32, category: String, value: f64, timestamp: String) -> Self {
        Self {
            id,
            category,
            value,
            timestamp,
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
        let reader = BufReader::new(file);
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        for result in csv_reader.records() {
            let record = result?;
            let id: u32 = record[0].parse()?;
            let category = record[1].to_string();
            let value: f64 = record[2].parse()?;
            let timestamp = record[3].to_string();

            self.records.push(DataRecord::new(id, category, value, timestamp));
        }

        Ok(())
    }

    fn filter_by_category(&self, category_filter: &str) -> Vec<&DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category_filter)
            .collect()
    }

    fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn save_filtered_to_csv(&self, filtered_records: Vec<&DataRecord>, output_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(output_path)?;
        let writer = BufWriter::new(file);
        let mut csv_writer = WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        csv_writer.write_record(&["ID", "Category", "Value", "Timestamp"])?;

        for record in filtered_records {
            csv_writer.write_record(&[
                record.id.to_string(),
                record.category.clone(),
                record.value.to_string(),
                record.timestamp.clone(),
            ])?;
        }

        csv_writer.flush()?;
        Ok(())
    }
}

fn process_data_sample() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input_data.csv")?;
    
    println!("Total records loaded: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Maximum value record: {:?}", max_record);
    }
    
    let filtered = processor.filter_by_category("electronics");
    println!("Filtered electronics records: {}", filtered.len());
    
    processor.save_filtered_to_csv(filtered, "filtered_output.csv")?;
    
    Ok(())
}