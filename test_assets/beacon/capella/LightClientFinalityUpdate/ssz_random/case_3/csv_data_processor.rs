
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

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
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| record.get(column_index) == Some(&value.to_string()))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;

        let sum: f64 = self.records
            .iter()
            .filter_map(|record| record.get(column_index))
            .filter_map(|value| value.parse::<f64>().ok())
            .sum();

        Some(sum)
    }

    pub fn group_by_column(&self, group_column: &str, aggregate_column: &str) -> HashMap<String, f64> {
        let group_idx = match self.headers.iter().position(|h| h == group_column) {
            Some(idx) => idx,
            None => return HashMap::new(),
        };

        let agg_idx = match self.headers.iter().position(|h| h == aggregate_column) {
            Some(idx) => idx,
            None => return HashMap::new(),
        };

        let mut result = HashMap::new();
        for record in &self.records {
            if let (Some(group_val), Some(agg_val)) = (record.get(group_idx), record.get(agg_idx)) {
                if let Ok(num) = agg_val.parse::<f64>() {
                    *result.entry(group_val.clone()).or_insert(0.0) += num;
                }
            }
        }

        result
    }

    pub fn get_record_count(&self) -> usize {
        self.records.len()
    }

    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,department,salary").unwrap();
        writeln!(file, "Alice,Engineering,75000").unwrap();
        writeln!(file, "Bob,Marketing,65000").unwrap();
        writeln!(file, "Charlie,Engineering,80000").unwrap();
        writeln!(file, "Diana,Marketing,70000").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(processor.get_headers(), vec!["name", "department", "salary"]);
        assert_eq!(processor.get_record_count(), 4);
    }

    #[test]
    fn test_filter_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", "Engineering");
        assert_eq!(engineering_records.len(), 2);
    }

    #[test]
    fn test_aggregate_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let total_salary = processor.aggregate_numeric_column("salary").unwrap();
        assert_eq!(total_salary, 290000.0);
    }

    #[test]
    fn test_group_by_column() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::new(test_file.path().to_str().unwrap()).unwrap();
        
        let dept_salaries = processor.group_by_column("department", "salary");
        assert_eq!(dept_salaries.get("Engineering"), Some(&155000.0));
        assert_eq!(dept_salaries.get("Marketing"), Some(&135000.0));
    }
}