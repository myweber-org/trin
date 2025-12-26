
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Record {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
}

pub struct CsvProcessor {
    records: Vec<Record>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Record = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<Record> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_average(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.records.iter().map(|r| r.value).sum();
        sum / self.records.len() as f64
    }

    pub fn find_max_value(&self) -> Option<&Record> {
        self.records.iter().max_by(|a, b| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn export_to_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let json_data = serde_json::to_string_pretty(&self.records)?;
        std::fs::write(path, json_data)?;
        Ok(())
    }

    pub fn add_record(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_empty_processor() {
        let processor = CsvProcessor::new();
        assert_eq!(processor.record_count(), 0);
        assert_eq!(processor.calculate_average(), 0.0);
    }

    #[test]
    fn test_record_operations() {
        let mut processor = CsvProcessor::new();
        
        let record = Record {
            id: 1,
            name: "Test".to_string(),
            value: 100.0,
            category: "A".to_string(),
        };

        processor.add_record(record);
        assert_eq!(processor.record_count(), 1);
        
        processor.clear();
        assert_eq!(processor.record_count(), 0);
    }

    #[test]
    fn test_filtering() {
        let mut processor = CsvProcessor::new();
        
        processor.add_record(Record {
            id: 1,
            name: "Item1".to_string(),
            value: 10.0,
            category: "Electronics".to_string(),
        });
        
        processor.add_record(Record {
            id: 2,
            name: "Item2".to_string(),
            value: 20.0,
            category: "Books".to_string(),
        });

        let filtered = processor.filter_by_category("Electronics");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Item1");
    }
}