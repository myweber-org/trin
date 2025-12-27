use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
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
            let line = line?;
            let fields: Vec<String> = line.split(',')
                .map(|s| s.trim().to_string())
                .collect();
            if fields.len() == headers.len() {
                records.push(fields);
            }
        }

        Ok(CsvProcessor { headers, records })
    }

    pub fn filter_by_column(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name);

        match column_index {
            Some(idx) => self.records.iter()
                .filter(|record| record.get(idx).map_or(false, |v| v == value))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn get_column_stats(&self, column_name: &str) -> Option<(usize, String, String)> {
        let column_index = self.headers.iter()
            .position(|h| h == column_name)?;

        let values: Vec<&str> = self.records.iter()
            .filter_map(|record| record.get(column_index).map(|s| s.as_str()))
            .collect();

        if values.is_empty() {
            return None;
        }

        let count = values.len();
        let min_value = values.iter().min()?.to_string();
        let max_value = values.iter().max()?.to_string();

        Some((count, min_value, max_value))
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn headers(&self) -> &[String] {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age,department").unwrap();
        writeln!(temp_file, "1,Alice,30,Engineering").unwrap();
        writeln!(temp_file, "2,Bob,25,Marketing").unwrap();
        writeln!(temp_file, "3,Charlie,35,Engineering").unwrap();
        writeln!(temp_file, "4,Diana,28,Sales").unwrap();
        temp_file
    }

    #[test]
    fn test_csv_loading() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.headers(), &["id", "name", "age", "department"]);
        assert_eq!(processor.record_count(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", "Engineering");
        assert_eq!(engineering_records.len(), 2);
        
        let sales_records = processor.filter_by_column("department", "Sales");
        assert_eq!(sales_records.len(), 1);
    }

    #[test]
    fn test_column_stats() {
        let temp_file = create_test_csv();
        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap()).unwrap();
        
        let age_stats = processor.get_column_stats("age").unwrap();
        assert_eq!(age_stats.0, 4);
        assert_eq!(age_stats.1, "25");
        assert_eq!(age_stats.2, "35");
    }
}