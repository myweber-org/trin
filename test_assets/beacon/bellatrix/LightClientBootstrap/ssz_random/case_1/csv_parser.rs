use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(std::io::Error),
    ParseError(String),
    InvalidHeader,
    MissingColumn,
}

impl From<std::io::Error> for CsvError {
    fn from(err: std::io::Error) -> Self {
        CsvError::IoError(err)
    }
}

pub struct CsvParser {
    delimiter: char,
    has_header: bool,
}

impl CsvParser {
    pub fn new() -> Self {
        CsvParser {
            delimiter: ',',
            has_header: true,
        }
    }

    pub fn with_delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    pub fn parse_file(&self, path: &str) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if self.has_header {
            lines.next();
        }

        let mut records = Vec::new();
        for (line_num, line) in lines.enumerate() {
            let line = line?;
            let record = self.parse_line(&line, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() < 4 {
            return Err(CsvError::MissingColumn);
        }

        let id = u32::from_str(parts[0])
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid ID - {}", line_num, e)))?;
        
        let name = parts[1].to_string();
        
        let value = f64::from_str(parts[2])
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid value - {}", line_num, e)))?;
        
        let active = bool::from_str(parts[3])
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid active flag - {}", line_num, e)))?;

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }
}

impl Default for CsvParser {
    fn default() -> Self {
        Self::new()
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
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "1,Test Item,42.5,true").unwrap();
        writeln!(temp_file, "2,Another Item,99.9,false").unwrap();

        let parser = CsvParser::new();
        let records = parser.parse_file(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].id, 1);
        assert_eq!(records[0].name, "Test Item");
        assert_eq!(records[0].value, 42.5);
        assert_eq!(records[0].active, true);
    }

    #[test]
    fn test_invalid_data() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "id,name,value,active").unwrap();
        writeln!(temp_file, "invalid,Test,42.5,true").unwrap();

        let parser = CsvParser::new();
        let result = parser.parse_file(temp_file.path().to_str().unwrap());
        
        assert!(matches!(result, Err(CsvError::ParseError(_))));
    }
}