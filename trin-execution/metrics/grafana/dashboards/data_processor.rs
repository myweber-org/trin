
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    pub delimiter: char,
    pub has_headers: bool,
}

impl Default for DataProcessor {
    fn default() -> Self {
        DataProcessor {
            delimiter: ',',
            has_headers: true,
        }
    }
}

impl DataProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        DataProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn process_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        filter_predicate: Option<Box<dyn Fn(&[String]) -> bool>>,
    ) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.has_headers {
            let _headers = lines.next();
        }

        let mut records = Vec::new();

        for line_result in lines {
            let line = line_result?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if let Some(ref predicate) = filter_predicate {
                if predicate(&fields) {
                    records.push(fields);
                }
            } else {
                records.push(fields);
            }
        }

        Ok(records)
    }

    pub fn filter_numeric_greater_than(
        &self,
        records: &[Vec<String>],
        column_index: usize,
        threshold: f64,
    ) -> Vec<Vec<String>> {
        records
            .iter()
            .filter(|fields| {
                if let Some(value_str) = fields.get(column_index) {
                    if let Ok(value) = value_str.parse::<f64>() {
                        return value > threshold;
                    }
                }
                false
            })
            .cloned()
            .collect()
    }

    pub fn calculate_column_average(
        &self,
        records: &[Vec<String>],
        column_index: usize,
    ) -> Option<f64> {
        let mut sum = 0.0;
        let mut count = 0;

        for fields in records {
            if let Some(value_str) = fields.get(column_index) {
                if let Ok(value) = value_str.parse::<f64>() {
                    sum += value;
                    count += 1;
                }
            }
        }

        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_headers() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,salary").unwrap();
        writeln!(temp_file, "Alice,30,50000").unwrap();
        writeln!(temp_file, "Bob,25,45000").unwrap();
        writeln!(temp_file, "Charlie,35,60000").unwrap();

        let processor = DataProcessor::default();
        let result = processor.process_file(temp_file.path(), None).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["Alice", "30", "50000"]);
    }

    #[test]
    fn test_filter_numeric_greater_than() {
        let records = vec![
            vec!["A".to_string(), "100".to_string()],
            vec!["B".to_string(), "50".to_string()],
            vec!["C".to_string(), "150".to_string()],
        ];

        let processor = DataProcessor::default();
        let filtered = processor.filter_numeric_greater_than(&records, 1, 75.0);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|r| r[0] == "A"));
        assert!(filtered.iter().any(|r| r[0] == "C"));
    }

    #[test]
    fn test_calculate_column_average() {
        let records = vec![
            vec!["10".to_string()],
            vec!["20".to_string()],
            vec!["30".to_string()],
        ];

        let processor = DataProcessor::default();
        let average = processor.calculate_column_average(&records, 0);

        assert_eq!(average, Some(20.0));
    }
}