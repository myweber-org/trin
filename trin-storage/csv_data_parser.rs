use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvParser {
    delimiter: char,
    has_headers: bool,
}

impl CsvParser {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvParser {
            delimiter,
            has_headers,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines().enumerate();

        if self.has_headers {
            let _ = lines.next();
        }

        for (line_num, line) in lines {
            let line = line?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            
            if record.iter().any(|field| field.is_empty()) {
                return Err(format!("Empty field detected at line {}", line_num + 1).into());
            }
            
            records.push(record);
        }

        if records.is_empty() {
            return Err("No data records found in CSV file".into());
        }

        Ok(records)
    }

    pub fn get_column(&self, data: &[Vec<String>], column_index: usize) -> Result<Vec<String>, Box<dyn Error>> {
        if data.is_empty() {
            return Err("No data available".into());
        }

        let expected_len = data[0].len();
        if column_index >= expected_len {
            return Err(format!("Column index {} out of bounds (max {})", column_index, expected_len - 1).into());
        }

        let column_data: Vec<String> = data
            .iter()
            .map(|row| {
                if row.len() != expected_len {
                    return Err(format!("Inconsistent row length: expected {}, found {}", expected_len, row.len()));
                }
                Ok(row[column_index].clone())
            })
            .collect::<Result<Vec<String>, String>>()?;

        Ok(column_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_csv_with_headers() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "New York"]);
        assert_eq!(result[1], vec!["Bob", "25", "London"]);
    }

    #[test]
    fn test_get_column() {
        let data = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];
        
        let parser = CsvParser::new(',', false);
        let column = parser.get_column(&data, 1).unwrap();
        
        assert_eq!(column, vec!["30", "25"]);
    }

    #[test]
    fn test_empty_field_error() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,,New York").unwrap();

        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path());
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty field"));
    }
}