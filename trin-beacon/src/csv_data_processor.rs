
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct CsvProcessor {
    records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, filepath: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let mut rdr = csv::Reader::from_reader(reader);

        for result in rdr.deserialize() {
            let record: CsvRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|record| record.value).sum()
    }

    pub fn calculate_average_value(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        self.calculate_total_value() / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&CsvRecord> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn save_filtered_to_file(
        &self,
        filtered_records: &[CsvRecord],
        output_path: &str,
    ) -> Result<(), Box<dyn Error>> {
        let file = File::create(output_path)?;
        let writer = BufWriter::new(file);
        let mut wtr = csv::Writer::from_writer(writer);

        for record in filtered_records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn add_record(&mut self, record: CsvRecord) {
        self.records.push(record);
    }

    pub fn clear_records(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processor_operations() {
        let mut processor = CsvProcessor::new();

        processor.add_record(CsvRecord {
            id: 1,
            name: "Item1".to_string(),
            category: "Electronics".to_string(),
            value: 100.0,
            active: true,
        });

        processor.add_record(CsvRecord {
            id: 2,
            name: "Item2".to_string(),
            category: "Books".to_string(),
            value: 50.0,
            active: false,
        });

        processor.add_record(CsvRecord {
            id: 3,
            name: "Item3".to_string(),
            category: "Electronics".to_string(),
            value: 200.0,
            active: true,
        });

        assert_eq!(processor.get_record_count(), 3);
        assert_eq!(processor.calculate_total_value(), 350.0);
        assert!((processor.calculate_average_value() - 116.666).abs() < 0.001);

        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);

        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 2);

        let max_record = processor.find_max_value().unwrap();
        assert_eq!(max_record.id, 3);
        assert_eq!(max_record.value, 200.0);

        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path().to_str().unwrap();

        processor
            .save_filtered_to_file(&electronics, output_path)
            .unwrap();

        let mut new_processor = CsvProcessor::new();
        new_processor.load_from_file(output_path).unwrap();
        assert_eq!(new_processor.get_record_count(), 2);
    }
}