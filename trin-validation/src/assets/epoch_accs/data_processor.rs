use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    delimiter: char,
    has_header: bool,
}

impl DataProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        DataProcessor {
            delimiter,
            has_header,
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

        let mut records = Vec::new();

        if self.has_header {
            if let Some(Ok(header_line)) = lines.next() {
                let headers: Vec<String> = header_line
                    .split(self.delimiter)
                    .map(|s| s.trim().to_string())
                    .collect();
                records.push(headers);
            }
        }

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
        records: &[Vec<String>],
        column_index: usize,
        threshold: f64,
    ) -> Vec<Vec<String>> {
        records
            .iter()
            .filter(|row| {
                if let Some(cell) = row.get(column_index) {
                    if let Ok(value) = cell.parse::<f64>() {
                        return value > threshold;
                    }
                }
                false
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_file_with_header() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,score").unwrap();
        writeln!(temp_file, "Alice,25,85.5").unwrap();
        writeln!(temp_file, "Bob,30,92.0").unwrap();
        writeln!(temp_file, "Charlie,22,78.5").unwrap();

        let processor = DataProcessor::new(',', true);
        let result = processor.process_file(temp_file.path(), None).unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], vec!["name", "age", "score"]);
        assert_eq!(result[1], vec!["Alice", "25", "85.5"]);
    }

    #[test]
    fn test_filter_numeric_greater_than() {
        let data = vec![
            vec!["A".to_string(), "10.5".to_string()],
            vec!["B".to_string(), "5.2".to_string()],
            vec!["C".to_string(), "15.8".to_string()],
        ];

        let filtered = DataProcessor::filter_numeric_greater_than(&data, 1, 10.0);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|row| row[0] == "A"));
        assert!(filtered.iter().any(|row| row[0] == "C"));
    }
}