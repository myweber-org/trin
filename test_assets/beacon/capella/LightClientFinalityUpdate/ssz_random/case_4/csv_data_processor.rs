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

    pub fn aggregate_numeric_column(&self, column_name: &str, operation: &str) -> Option<f64> {
        let column_index = self.headers.iter().position(|h| h == column_name)?;
        
        let numeric_values: Vec<f64> = self.records
            .iter()
            .filter_map(|record| record[column_index].parse().ok())
            .collect();

        if numeric_values.is_empty() {
            return None;
        }

        match operation {
            "sum" => Some(numeric_values.iter().sum()),
            "avg" => Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64),
            "max" => numeric_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            "min" => numeric_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied(),
            _ => None,
        }
    }

    pub fn group_by_column(&self, group_column: &str, agg_column: &str) -> HashMap<String, f64> {
        let group_idx = match self.headers.iter().position(|h| h == group_column) {
            Some(idx) => idx,
            None => return HashMap::new(),
        };

        let agg_idx = match self.headers.iter().position(|h| h == agg_column) {
            Some(idx) => idx,
            None => return HashMap::new(),
        };

        let mut groups: HashMap<String, Vec<f64>> = HashMap::new();
        
        for record in &self.records {
            if let (Some(group_val), Ok(agg_val)) = (
                record.get(group_idx),
                record.get(agg_idx).and_then(|v| v.parse().ok())
            ) {
                groups.entry(group_val.clone())
                    .or_insert_with(Vec::new)
                    .push(agg_val);
            }
        }

        groups.into_iter()
            .map(|(key, values)| (key, values.iter().sum::<f64>() / values.len() as f64))
            .collect()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn column_names(&self) -> &[String] {
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
        writeln!(file, "name,age,salary,department").unwrap();
        writeln!(file, "Alice,30,50000,Engineering").unwrap();
        writeln!(file, "Bob,25,45000,Sales").unwrap();
        writeln!(file, "Charlie,35,60000,Engineering").unwrap();
        writeln!(file, "Diana,28,48000,Marketing").unwrap();
        file
    }

    #[test]
    fn test_csv_loading() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(processor.record_count(), 4);
        assert_eq!(processor.column_names(), &["name", "age", "salary", "department"]);
    }

    #[test]
    fn test_filtering() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        let engineering_records = processor.filter_by_column("department", |dept| dept == "Engineering");
        assert_eq!(engineering_records.len(), 2);
        
        let high_salary = processor.filter_by_column("salary", |sal| sal.parse::<i32>().unwrap_or(0) > 55000);
        assert_eq!(high_salary.len(), 1);
    }

    #[test]
    fn test_aggregation() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        let avg_age = processor.aggregate_numeric_column("age", "avg").unwrap();
        assert!((avg_age - 29.5).abs() < 0.001);
        
        let max_salary = processor.aggregate_numeric_column("salary", "max").unwrap();
        assert_eq!(max_salary, 60000.0);
    }

    #[test]
    fn test_grouping() {
        let file = create_test_csv();
        let processor = CsvProcessor::from_file(file.path().to_str().unwrap()).unwrap();
        
        let dept_avg_salary = processor.group_by_column("department", "salary");
        assert_eq!(dept_avg_salary.get("Engineering").unwrap(), &55000.0);
        assert_eq!(dept_avg_salary.get("Sales").unwrap(), &45000.0);
    }
}