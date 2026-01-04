use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    headers: Vec<String>,
    records: Vec<Vec<String>>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            headers: Vec::new(),
            records: Vec::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(first_line) = lines.next() {
            self.headers = first_line?
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        for line_result in lines {
            let line = line_result?;
            if line.trim().is_empty() {
                continue;
            }
            let record: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            self.records.push(record);
        }

        Ok(())
    }

    pub fn filter_by_column_value(&self, column_name: &str, value: &str) -> Vec<Vec<String>> {
        let column_index = match self.headers.iter().position(|h| h == column_name) {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        self.records
            .iter()
            .filter(|record| {
                record.get(column_index)
                    .map(|cell| cell == value)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub fn get_column_names(&self) -> &Vec<String> {
        &self.headers
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,age").unwrap();
        writeln!(temp_file, "1,Alice,30").unwrap();
        writeln!(temp_file, "2,Bob,25").unwrap();
        writeln!(temp_file, "3,Charlie,30").unwrap();

        let mut processor = CsvProcessor::new();
        processor.load_from_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(processor.record_count(), 3);
        assert_eq!(processor.get_column_names(), &vec!["id", "name", "age"]);

        let filtered = processor.filter_by_column_value("age", "30");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0], vec!["1", "Alice", "30"]);
        assert_eq!(filtered[1], vec!["3", "Charlie", "30"]);
    }
}