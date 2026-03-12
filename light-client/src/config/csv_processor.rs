use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct CsvProcessor {
    file_path: String,
    delimiter: char,
}

impl CsvProcessor {
    pub fn new(file_path: &str, delimiter: char) -> Self {
        CsvProcessor {
            file_path: file_path.to_string(),
            delimiter,
        }
    }

    pub fn read_and_transform(&self) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut transformed_data = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_uppercase())
                .collect();
            
            if !parts.is_empty() && !parts.iter().all(|s| s.is_empty()) {
                transformed_data.push(parts);
            }
        }

        Ok(transformed_data)
    }

    pub fn filter_by_column(&self, data: &[Vec<String>], column_index: usize, filter_value: &str) -> Vec<Vec<String>> {
        data.iter()
            .filter(|row| {
                row.get(column_index)
                    .map(|value| value.contains(filter_value))
                    .unwrap_or(false)
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
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "alice,30,new york").unwrap();
        writeln!(temp_file, "bob,25,london").unwrap();

        let processor = CsvProcessor::new(temp_file.path().to_str().unwrap(), ',');
        let result = processor.read_and_transform().unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][0], "ALICE");
        assert_eq!(result[1][2], "LONDON");

        let filtered = processor.filter_by_column(&result, 2, "LONDON");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0][1], "25");
    }
}