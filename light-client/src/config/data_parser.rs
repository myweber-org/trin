use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

    pub fn parse_file(&self, file_path: &str) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines().enumerate();

        if self.has_headers {
            lines.next();
        }

        for (line_num, line_result) in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|field| field.trim().to_string())
                .collect();

            if !record.is_empty() {
                records.push(record);
            } else {
                eprintln!("Warning: Empty record at line {}", line_num + 1);
            }
        }

        Ok(records)
    }

    pub fn get_column(&self, data: &[Vec<String>], column_index: usize) -> Result<Vec<String>, String> {
        if data.is_empty() {
            return Err("No data available".to_string());
        }

        let mut column_data = Vec::new();
        for (row_index, row) in data.iter().enumerate() {
            if column_index < row.len() {
                column_data.push(row[column_index].clone());
            } else {
                return Err(format!("Column index {} out of bounds at row {}", column_index, row_index));
            }
        }

        Ok(column_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path().to_str().unwrap());

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_get_column() {
        let test_data = vec![
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec!["d".to_string(), "e".to_string(), "f".to_string()],
        ];

        let parser = CsvParser::new(',', false);
        let column = parser.get_column(&test_data, 1).unwrap();
        assert_eq!(column, vec!["b", "e"]);
    }
}