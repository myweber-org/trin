use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let headers = if let Some(first_line) = lines.next() {
            first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        } else {
            return Err("Empty CSV file".into());
        };

        let mut records = Vec::new();
        for line in lines {
            let record: Vec<String> = line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if record.len() == headers.len() {
                records.push(record);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = self.headers.iter().position(|h| h == column_name);
        
        column_index.map_or_else(Vec::new, |idx| {
            self.records
                .iter()
                .filter(|record| record.get(idx).map_or(false, |v| v == value))
                .cloned()
                .collect()
        })
    }

    pub fn get_column_names(&self) -> &[String] {
        &self.headers
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_unique_values(&self, column_name: &str) -> Vec<String> {
        let column_index = self.headers.iter().position(|h| h == column_name);
        
        column_index.map_or_else(Vec::new, |idx| {
            let mut values: Vec<String> = self.records
                .iter()
                .filter_map(|record| record.get(idx).cloned())
                .collect();
            
            values.sort();
            values.dedup();
            values
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "id,name,age,department").unwrap();
        writeln!(file, "1,Alice,30,Engineering").unwrap();
        writeln!(file, "2,Bob,25,Marketing").unwrap();
        writeln!(file, "3,Charlie,30,Engineering").unwrap();
        writeln!(file, "4,Diana,28,Marketing").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path()).unwrap();
        
        assert_eq!(processor.get_column_names(), &["id", "name", "age", "department"]);
        assert_eq!(processor.record_count(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", "Engineering");
        assert_eq!(engineering_records.len(), 2);
        
        let marketing_records = processor.filter_by_column("department", "Marketing");
        assert_eq!(marketing_records.len(), 2);
    }

    #[test]
    fn test_get_unique_values() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path()).unwrap();
        
        let departments = processor.get_unique_values("department");
        assert_eq!(departments, vec!["Engineering", "Marketing"]);
        
        let ages = processor.get_unique_values("age");
        assert_eq!(ages, vec!["25", "28", "30"]);
    }
}