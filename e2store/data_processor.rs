use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct DataProcessor {
    file_path: String,
    delimiter: char,
}

impl DataProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        DataProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn load_data(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let path = Path::new(&self.file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let fields: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            records.push(fields);
        }

        Ok(records)
    }

    pub fn filter_by_column(&self, column_index: usize, filter_value: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let data = self.load_data()?;
        let filtered: Vec<Vec<String>> = data
            .into_iter()
            .filter(|row| {
                if let Some(value) = row.get(column_index) {
                    value == filter_value
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered)
    }

    pub fn get_column_stats(&self, column_index: usize) -> Result<(usize, Option<String>, Option<String>), Box<dyn Error>> {
        let data = self.load_data()?;
        let mut values: Vec<&String> = Vec::new();

        for row in &data {
            if let Some(value) = row.get(column_index) {
                values.push(value);
            }
        }

        let count = values.len();
        let min = values.iter().min().map(|s| s.to_string());
        let max = values.iter().max().map(|s| s.to_string());

        Ok((count, min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_data_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let result = processor.load_data();
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 2);
    }

    #[test]
    fn test_filtering() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,30,Paris").unwrap();

        let processor = DataProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let filtered = processor.filter_by_column(1, "30").unwrap();
        assert_eq!(filtered.len(), 2);
    }
}