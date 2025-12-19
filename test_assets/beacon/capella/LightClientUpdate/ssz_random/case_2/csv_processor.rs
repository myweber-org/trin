use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
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

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines();

        if self.has_header {
            let _header = lines.next().ok_or(CsvError::InvalidHeader)??;
        }

        for (line_num, line) in lines.enumerate() {
            let line_content = line?;
            let record = self.parse_line(&line_content, line_num + 1)?;
            records.push(record);
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_num: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();
        
        if parts.len() < 3 {
            return Err(CsvError::MissingColumn);
        }

        let id = parts[0]
            .parse::<u32>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid ID - {}", line_num, e)))?;
        
        let name = parts[1].trim().to_string();
        
        let value = parts[2]
            .parse::<f64>()
            .map_err(|e| CsvError::ParseError(format!("Line {}: Invalid value - {}", line_num, e)))?;

        Ok(CsvRecord { id, name, value })
    }

    pub fn calculate_stats(records: &[CsvRecord]) -> (f64, f64, f64) {
        if records.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len() as f64;
        let mean = sum / count;

        let variance: f64 = records.iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>() / count;
        
        let std_dev = variance.sqrt();

        (mean, variance, std_dev)
    }
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CsvError::InvalidHeader => write!(f, "Invalid or missing CSV header"),
            CsvError::MissingColumn => write!(f, "Missing required columns in CSV"),
        }
    }
}

impl Error for CsvError {}