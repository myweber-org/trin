
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub category: String,
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

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: CsvRecord = result?;
            self.validate_record(&record)?;
            self.records.push(record);
        }

        Ok(())
    }

    fn validate_record(&self, record: &CsvRecord) -> Result<(), String> {
        if record.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        
        if record.value < 0.0 {
            return Err("Value cannot be negative".to_string());
        }
        
        if record.category.is_empty() {
            return Err("Category cannot be empty".to_string());
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<CsvRecord> {
        self.records
            .iter()
            .filter(|r| r.category == category)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records.iter().map(|r| r.value).sum()
    }

    pub fn get_average_value(&self) -> f64 {
        if self.records.is_empty() {
            0.0
        } else {
            self.calculate_total_value() / self.records.len() as f64
        }
    }

    pub fn transform_values<F>(&mut self, transform_fn: F)
    where
        F: Fn(f64) -> f64,
    {
        for record in &mut self.records {
            record.value = transform_fn(record.value);
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = csv::Writer::from_writer(file);

        for record in &self.records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn clear(&mut self) {
        self.records.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processor() {
        let mut processor = CsvProcessor::new();
        assert_eq!(processor.get_record_count(), 0);
        
        let test_data = vec![
            CsvRecord { id: 1, name: "Item1".to_string(), value: 10.5, category: "A".to_string() },
            CsvRecord { id: 2, name: "Item2".to_string(), value: 20.0, category: "B".to_string() },
        ];
        
        processor.records = test_data;
        assert_eq!(processor.get_record_count(), 2);
        assert_eq!(processor.calculate_total_value(), 30.5);
        assert_eq!(processor.get_average_value(), 15.25);
    }

    #[test]
    fn test_filter_by_category() {
        let mut processor = CsvProcessor::new();
        processor.records = vec![
            CsvRecord { id: 1, name: "Test1".to_string(), value: 10.0, category: "CategoryA".to_string() },
            CsvRecord { id: 2, name: "Test2".to_string(), value: 20.0, category: "CategoryB".to_string() },
            CsvRecord { id: 3, name: "Test3".to_string(), value: 30.0, category: "CategoryA".to_string() },
        ];
        
        let filtered = processor.filter_by_category("CategoryA");
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|r| r.category == "CategoryA"));
    }

    #[test]
    fn test_transform_values() {
        let mut processor = CsvProcessor::new();
        processor.records = vec![
            CsvRecord { id: 1, name: "Item".to_string(), value: 10.0, category: "Test".to_string() },
        ];
        
        processor.transform_values(|v| v * 2.0);
        assert_eq!(processor.records[0].value, 20.0);
    }

    #[test]
    fn test_file_operations() -> Result<(), Box<dyn Error>> {
        let mut processor = CsvProcessor::new();
        processor.records = vec![
            CsvRecord { id: 1, name: "TestItem".to_string(), value: 15.5, category: "Test".to_string() },
        ];
        
        let temp_file = NamedTempFile::new()?;
        let path = temp_file.path();
        
        processor.save_to_file(path)?;
        
        let mut new_processor = CsvProcessor::new();
        new_processor.load_from_file(path)?;
        
        assert_eq!(new_processor.get_record_count(), 1);
        assert_eq!(new_processor.records[0].name, "TestItem");
        
        Ok(())
    }
}