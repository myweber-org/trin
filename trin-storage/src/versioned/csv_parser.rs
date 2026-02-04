use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl CsvParser {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvParser {
            delimiter,
            has_header,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        for line_result in lines {
            let line = line_result?;
            let record: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();
            if !record.is_empty() {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn parse_string(&self, content: &str) -> Vec<Vec<String>> {
        content
            .lines()
            .skip(if self.has_header { 1 } else { 0 })
            .map(|line| {
                line.split(self.delimiter)
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .filter(|record: &Vec<String>| !record.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_comma_delimited() {
        let parser = CsvParser::new(',', true);
        let csv_data = "name,age,city\nJohn,30,New York\nJane,25,London";
        let result = parser.parse_string(csv_data);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["John", "30", "New York"]);
    }

    #[test]
    fn test_parse_semicolon_delimited() {
        let parser = CsvParser::new(';', false);
        let csv_data = "Apple;1.99;5\nBanana;0.99;12";
        let result = parser.parse_string(csv_data);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Apple", "1.99", "5"]);
    }

    #[test]
    fn test_parse_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,value,status").unwrap();
        writeln!(temp_file, "1,100,active").unwrap();
        writeln!(temp_file, "2,200,inactive").unwrap();

        let parser = CsvParser::new(',', true);
        let result = parser.parse_file(temp_file.path()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["1", "100", "active"]);
    }

    #[test]
    fn test_empty_fields() {
        let parser = CsvParser::new(',', false);
        let csv_data = "a,b,c\n1,,3\n4,5,";
        let result = parser.parse_string(csv_data);
        assert_eq!(result[0], vec!["a", "b", "c"]);
        assert_eq!(result[1], vec!["1", "", "3"]);
        assert_eq!(result[2], vec!["4", "5", ""]);
    }
}