use csv::{ReaderBuilder, StringRecord};
use std::error::Error;
use std::fs::File;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<StringRecord>,
}

impl CsvProcessor {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let headers = rdr
            .headers()?
            .iter()
            .map(|s| s.to_string())
            .collect();

        let records: Vec<StringRecord> = rdr.records().collect::<Result<_, _>>()?;

        if records.is_empty() {
            return Err("CSV file contains no data records".into());
        }

        Ok(Self { headers, records })
    }

    pub fn validate_column_count(&self) -> Result<(), String> {
        let expected_len = self.headers.len();
        
        for (i, record) in self.records.iter().enumerate() {
            if record.len() != expected_len {
                return Err(format!(
                    "Row {} has {} columns, expected {}",
                    i + 1,
                    record.len(),
                    expected_len
                ));
            }
        }
        Ok(())
    }

    pub fn get_column_stats(&self, column_index: usize) -> Result<(usize, usize), String> {
        if column_index >= self.headers.len() {
            return Err(format!("Column index {} out of bounds", column_index));
        }

        let mut max_len = 0;
        let mut min_len = usize::MAX;

        for record in &self.records {
            if let Some(field) = record.get(column_index) {
                let len = field.len();
                max_len = max_len.max(len);
                min_len = min_len.min(len);
            }
        }

        if min_len == usize::MAX {
            min_len = 0;
        }

        Ok((min_len, max_len))
    }

    pub fn print_summary(&self) {
        println!("CSV Summary:");
        println!("Headers: {}", self.headers.join(", "));
        println!("Total records: {}", self.records.len());
        println!("Columns: {}", self.headers.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,city").unwrap();
        writeln!(file, "Alice,30,New York").unwrap();
        writeln!(file, "Bob,25,London").unwrap();
        writeln!(file, "Charlie,35,Tokyo").unwrap();
        file
    }

    #[test]
    fn test_csv_processing() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.headers.len(), 3);
        assert_eq!(processor.records.len(), 3);
        
        let validation = processor.validate_column_count();
        assert!(validation.is_ok());
        
        let stats = processor.get_column_stats(0).unwrap();
        assert_eq!(stats, (5, 7));
    }

    #[test]
    fn test_invalid_column_index() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        
        let result = processor.get_column_stats(10);
        assert!(result.is_err());
    }
}