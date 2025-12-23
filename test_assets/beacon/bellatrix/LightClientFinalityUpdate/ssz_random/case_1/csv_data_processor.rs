use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
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

    pub fn filter_by_column(&self, column_name: &str, predicate: impl Fn(&str) -> bool) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| predicate(&record[column_index]))
            .cloned()
            .collect()
    }

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Result<f64, String> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Err(format!("Column '{}' not found", column_name)),
        };

        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if numeric_values.is_empty() {
            return Err("No valid numeric values found".into());
        }

        match operation {
            "sum" => Ok(numeric_values.iter().sum()),
            "avg" => Ok(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "max" => Ok(numeric_values
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(b))),
            "min" => Ok(numeric_values
                .iter()
                .fold(f64::INFINITY, |a, &b| a.min(b))),
            _ => Err(format!("Unsupported operation: {}", operation)),
        }
    }

    pub fn group_by_column(&self, group_column: &str, agg_column: &str) -> Result<HashMap<String, f64>, String> {
        let group_idx = match self.headers.iter().position(|h| h == group_column) {
            Some(idx) => idx,
            None => return Err(format!("Group column '{}' not found", group_column)),
        };

        let agg_idx = match self.headers.iter().position(|h| h == agg_column) {
            Some(idx) => idx,
            None => return Err(format!("Aggregation column '{}' not found", agg_column)),
        };

        let mut groups: HashMap<String, Vec<f64>> = HashMap::new();

        for record in &self.records {
            if let (Some(group_val), Ok(agg_val)) = (record.get(group_idx), record[agg_idx].parse::<f64>()) {
                groups.entry(group_val.clone()).or_default().push(agg_val);
            }
        }

        let result: HashMap<String, f64> = groups
            .into_iter()
            .map(|(key, values)| {
                let avg = values.iter().sum::<f64>() / values.len() as f64;
                (key, avg)
            })
            .collect();

        Ok(result)
    }

    pub fn get_summary(&self) -> (usize, usize) {
        (self.headers.len(), self.records.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,salary").unwrap();
        writeln!(file, "Alice,30,50000").unwrap();
        writeln!(file, "Bob,25,45000").unwrap();
        writeln!(file, "Charlie,35,60000").unwrap();
        writeln!(file, "David,30,55000").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        let (cols, rows) = processor.get_summary();
        assert_eq!(cols, 3);
        assert_eq!(rows, 4);
    }

    #[test]
    fn test_filtering() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        let filtered = processor.filter_by_column("age", |age| age == "30");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_aggregation() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        let total_salary = processor.aggregate_numeric_column("salary", "sum").unwrap();
        assert_eq!(total_salary, 210000.0);
    }

    #[test]
    fn test_grouping() {
        let test_file = create_test_csv();
        let processor = CsvProcessor::from_file(test_file.path().to_str().unwrap()).unwrap();
        let groups = processor.group_by_column("age", "salary").unwrap();
        assert_eq!(groups.get("30").unwrap(), &52500.0);
    }
}