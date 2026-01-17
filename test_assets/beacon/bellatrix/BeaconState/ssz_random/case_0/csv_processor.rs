use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn validate_file<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if index == 0 && self.has_headers {
                column_count = Some(columns.len());
                continue;
            }

            if let Some(expected) = column_count {
                if columns.len() != expected {
                    return Err(format!("Line {} has {} columns, expected {}", 
                        index + 1, columns.len(), expected).into());
                }
            } else {
                column_count = Some(columns.len());
            }
            
            line_count += 1;
        }

        Ok(line_count)
    }

    pub fn extract_column<P: AsRef<Path>>(&self, file_path: P, column_index: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut result = Vec::new();
        let mut start_line = 0;

        if self.has_headers {
            start_line = 1;
        }

        for (index, line) in reader.lines().enumerate() {
            if index < start_line {
                continue;
            }

            let line = line?;
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if column_index < columns.len() {
                result.push(columns[column_index].to_string());
            } else {
                return Err(format!("Column index {} out of bounds on line {}", 
                    column_index, index + 1).into());
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_file(temp_file.path());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_column_extraction() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let processor = CsvProcessor::new(',', true);
        let column = processor.extract_column(temp_file.path(), 0).unwrap();
        assert_eq!(column, vec!["Alice", "Bob"]);
    }
}