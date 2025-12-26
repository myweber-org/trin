use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseFloatError;

#[derive(Debug)]
enum CsvError {
    IoError(std::io::Error),
    ParseError(ParseFloatError),
    InvalidColumnCount(usize, usize),
    EmptyFile,
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(e) => write!(f, "Parse error: {}", e),
            CsvError::InvalidColumnCount(expected, actual) => {
                write!(f, "Invalid column count: expected {}, got {}", expected, actual)
            }
            CsvError::EmptyFile => write!(f, "CSV file is empty"),
        }
    }
}

impl Error for CsvError {}

impl From<std::io::Error> for CsvError {
    fn from(error: std::io::Error) -> Self {
        CsvError::IoError(error)
    }
}

impl From<ParseFloatError> for CsvError {
    fn from(error: ParseFloatError) -> Self {
        CsvError::ParseError(error)
    }
}

struct CsvRecord {
    id: u32,
    value: f64,
    category: String,
}

impl CsvRecord {
    fn from_line(line: &str, expected_columns: usize) -> Result<Self, CsvError> {
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() != expected_columns {
            return Err(CsvError::InvalidColumnCount(expected_columns, parts.len()));
        }
        
        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError("0.0".parse::<f64>().unwrap_err())
        })?;
        
        let value = parts[1].parse::<f64>()?;
        let category = parts[2].to_string();
        
        Ok(CsvRecord { id, value, category })
    }
}

fn process_csv_file(file_path: &str) -> Result<Vec<CsvRecord>, CsvError> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut line_count = 0;
    
    for line_result in reader.lines() {
        let line = line_result?;
        line_count += 1;
        
        if line.trim().is_empty() {
            continue;
        }
        
        let record = CsvRecord::from_line(&line, 3)?;
        records.push(record);
    }
    
    if line_count == 0 {
        return Err(CsvError::EmptyFile);
    }
    
    Ok(records)
}

fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, f64) {
    let count = records.len() as f64;
    let sum: f64 = records.iter().map(|r| r.value).sum();
    let mean = sum / count;
    
    let variance: f64 = records.iter()
        .map(|r| (r.value - mean).powi(2))
        .sum::<f64>() / count;
    
    let std_dev = variance.sqrt();
    
    (sum, mean, std_dev)
}

fn main() {
    let file_path = "data.csv";
    
    match process_csv_file(file_path) {
        Ok(records) => {
            println!("Successfully processed {} records", records.len());
            
            let (sum, mean, std_dev) = calculate_statistics(&records);
            println!("Sum: {:.2}, Mean: {:.2}, Std Dev: {:.2}", sum, mean, std_dev);
            
            for record in records.iter().take(3) {
                println!("ID: {}, Value: {:.2}, Category: {}", 
                         record.id, record.value, record.category);
            }
        }
        Err(e) => {
            eprintln!("Error processing CSV: {}", e);
        }
    }
}