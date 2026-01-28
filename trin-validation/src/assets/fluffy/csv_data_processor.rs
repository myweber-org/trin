use std::error::Error;
use std::fs::File;
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
struct DataRecord {
    id: u32,
    category: String,
    value: f64,
    active: bool,
}

impl DataRecord {
    fn new(id: u32, category: &str, value: f64, active: bool) -> Self {
        DataRecord {
            id,
            category: category.to_string(),
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
        let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

        for result in rdr.records() {
            let record = result?;
            let id: u32 = record[0].parse()?;
            let category = record[1].to_string();
            let value: f64 = record[2].parse()?;
            let active: bool = record[3].parse()?;

            self.records.push(DataRecord::new(id, &category, value, active));
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

        let sum: f64 = self.records.iter().map(|record| record.value).sum();
        sum / self.records.len() as f64
    }

    fn find_max_value(&self) -> Option<&DataRecord> {
        self.records.iter().max_by(|a, b| {
            a.value.partial_cmp(&b.value).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    fn save_filtered_to_csv(&self, file_path: &str, category: &str) -> Result<(), Box<dyn Error>> {
        let filtered = self.filter_by_category(category);
        let file = File::create(file_path)?;
        let mut wtr = WriterBuilder::new().from_writer(file);

        wtr.write_record(&["id", "category", "value", "active"])?;

        for record in filtered {
            wtr.write_record(&[
                record.id.to_string(),
                record.category.clone(),
                record.value.to_string(),
                record.active.to_string(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut processor = DataProcessor::new();
    
    processor.load_from_csv("input_data.csv")?;
    
    println!("Total records loaded: {}", processor.records.len());
    println!("Average value: {:.2}", processor.calculate_average());
    
    if let Some(max_record) = processor.find_max_value() {
        println!("Max value record: ID {}, Value {}", max_record.id, max_record.value);
    }
    
    let active_records = processor.filter_active();
    println!("Active records: {}", active_records.len());
    
    processor.save_filtered_to_csv("filtered_data.csv", "electronics")?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_record_creation() {
        let record = DataRecord::new(1, "electronics", 99.99, true);
        assert_eq!(record.id, 1);
        assert_eq!(record.category, "electronics");
        assert_eq!(record.value, 99.99);
        assert_eq!(record.active, true);
    }

    #[test]
    fn test_empty_average() {
        let processor = DataProcessor::new();
        assert_eq!(processor.calculate_average(), 0.0);
    }

    #[test]
    fn test_filtering() {
        let mut processor = DataProcessor::new();
        processor.records.push(DataRecord::new(1, "electronics", 100.0, true));
        processor.records.push(DataRecord::new(2, "clothing", 50.0, false));
        
        let filtered = processor.filter_by_category("electronics");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
    }
}