use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 && self.has_headers {
                continue;
            }

            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !fields.is_empty() {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn write_file<P: AsRef<Path>>(
        &self,
        path: P,
        data: &[Vec<String>],
        headers: Option<&[String]>,
    ) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;

        if let Some(headers) = headers {
            let header_line = headers.join(&self.delimiter.to_string());
            writeln!(file, "{}", header_line)?;
        }

        for record in data {
            let line = record.join(&self.delimiter.to_string());
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }

    pub fn filter_records<F>(&self, records: &[Vec<String>], predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }

    pub fn extract_column(&self, records: &[Vec<String>], column_index: usize) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| record.get(column_index).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let processor = CsvProcessor::new(',', true);
        
        let test_data = vec![
            vec!["name".to_string(), "age".to_string(), "city".to_string()],
            vec!["Alice".to_string(), "30".to_string(), "New York".to_string()],
            vec!["Bob".to_string(), "25".to_string(), "London".to_string()],
            vec!["Charlie".to_string(), "35".to_string(), "Paris".to_string()],
        ];

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        processor
            .write_file(path, &test_data[1..], Some(&test_data[0]))
            .unwrap();

        let mut file_content = String::new();
        File::open(path)
            .unwrap()
            .read_to_string(&mut file_content)
            .unwrap();

        assert!(file_content.contains("Alice,30,New York"));
        assert!(file_content.contains("name,age,city"));

        let read_data = processor.read_file(path).unwrap();
        assert_eq!(read_data.len(), 3);
        assert_eq!(read_data[0], vec!["Alice", "30", "New York"]);

        let filtered = processor.filter_records(&read_data, |record| {
            record.get(1).map_or(false, |age| age.parse::<i32>().unwrap_or(0) > 30)
        });
        
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], vec!["Charlie", "35", "Paris"]);

        let ages = processor.extract_column(&read_data, 1);
        assert_eq!(ages, vec!["30", "25", "35"]);
    }
}
use std::error::Error;
use std::fs::File;
use csv::{ReaderBuilder, WriterBuilder};

#[derive(Debug, Clone)]
pub struct DataRecord {
    pub id: u32,
    pub name: String,
    pub category: String,
    pub value: f64,
    pub active: bool,
}

pub struct DataProcessor {
    records: Vec<DataRecord>,
}

impl DataProcessor {
    pub fn new() -> Self {
        DataProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_csv(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        for result in rdr.deserialize() {
            let record: DataRecord = result?;
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.category == category)
            .cloned()
            .collect()
    }

    pub fn filter_active(&self) -> Vec<DataRecord> {
        self.records
            .iter()
            .filter(|record| record.active)
            .cloned()
            .collect()
    }

    pub fn calculate_total_value(&self) -> f64 {
        self.records
            .iter()
            .map(|record| record.value)
            .sum()
    }

    pub fn calculate_average_value(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        Some(self.calculate_total_value() / self.records.len() as f64)
    }

    pub fn find_max_value_record(&self) -> Option<DataRecord> {
        self.records
            .iter()
            .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap())
            .cloned()
    }

    pub fn save_filtered_to_csv(&self, file_path: &str, records: &[DataRecord]) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_path)?;
        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .from_writer(file);

        for record in records {
            wtr.serialize(record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
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
    fn test_data_processor_operations() {
        let mut processor = DataProcessor::new();
        
        let test_records = vec![
            DataRecord {
                id: 1,
                name: "Item A".to_string(),
                category: "Electronics".to_string(),
                value: 100.0,
                active: true,
            },
            DataRecord {
                id: 2,
                name: "Item B".to_string(),
                category: "Books".to_string(),
                value: 50.0,
                active: false,
            },
            DataRecord {
                id: 3,
                name: "Item C".to_string(),
                category: "Electronics".to_string(),
                value: 200.0,
                active: true,
            },
        ];

        processor.records = test_records;

        assert_eq!(processor.get_record_count(), 3);
        assert_eq!(processor.calculate_total_value(), 350.0);
        assert_eq!(processor.calculate_average_value(), Some(116.66666666666667));
        
        let electronics = processor.filter_by_category("Electronics");
        assert_eq!(electronics.len(), 2);
        
        let active_items = processor.filter_active();
        assert_eq!(active_items.len(), 2);
        
        let max_record = processor.find_max_value_record();
        assert!(max_record.is_some());
        assert_eq!(max_record.unwrap().value, 200.0);
    }

    #[test]
    fn test_csv_export() {
        let processor = DataProcessor::new();
        let temp_file = NamedTempFile::new().unwrap();
        let test_records = vec![
            DataRecord {
                id: 1,
                name: "Test Item".to_string(),
                category: "Test".to_string(),
                value: 10.0,
                active: true,
            },
        ];

        let result = processor.save_filtered_to_csv(temp_file.path().to_str().unwrap(), &test_records);
        assert!(result.is_ok());
    }
}