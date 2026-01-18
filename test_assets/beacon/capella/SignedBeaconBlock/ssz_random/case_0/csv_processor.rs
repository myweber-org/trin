
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn validate_file<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;

        for (index, line) in reader.lines().enumerate() {
            let line_content = line?;
            let columns: Vec<&str> = line_content.split(self.delimiter).collect();
            
            if let Some(expected_count) = column_count {
                if columns.len() != expected_count {
                    return Err(format!("Line {} has {} columns, expected {}", 
                        index + 1, columns.len(), expected_count).into());
                }
            } else {
                column_count = Some(columns.len());
            }
            
            line_count += 1;
        }

        if line_count == 0 {
            return Err("Empty CSV file".into());
        }

        Ok(line_count)
    }

    pub fn transform_column<P: AsRef<Path>>(
        &self, 
        file_path: P, 
        column_index: usize,
        transformer: fn(&str) -> String
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();
        let mut start_line = 0;

        if self.has_header {
            start_line = 1;
        }

        for (index, line) in reader.lines().enumerate() {
            if index < start_line {
                continue;
            }

            let line_content = line?;
            let columns: Vec<&str> = line_content.split(self.delimiter).collect();
            
            if column_index >= columns.len() {
                return Err(format!("Column index {} out of bounds on line {}", 
                    column_index, index + 1).into());
            }

            let transformed = transformer(columns[column_index]);
            results.push(transformed);
        }

        Ok(results)
    }
}

pub fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

pub fn trim_transform(value: &str) -> String {
    value.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "name,age,city").unwrap();
        writeln!(file, "john,25,new york").unwrap();
        writeln!(file, "jane,30,london").unwrap();
        file
    }

    #[test]
    fn test_validate_file() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_file(file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_transform_column() {
        let file = create_test_csv();
        let processor = CsvProcessor::new(',', true);
        let result = processor.transform_column(file.path(), 0, uppercase_transform);
        
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed, vec!["JOHN", "JANE"]);
    }
}