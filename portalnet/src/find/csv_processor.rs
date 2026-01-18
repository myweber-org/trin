use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
enum CsvError {
    IoError(std::io::Error),
    ParseError(String, usize),
    InvalidHeader(String),
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::IoError(e) => write!(f, "IO error: {}", e),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
        }
    }
}

impl Error for CsvError {}

impl From<std::io::Error> for CsvError {
    fn from(error: std::io::Error) -> Self {
        CsvError::IoError(error)
    }
}

struct CsvProcessor {
    headers: Vec<String>,
    data: Vec<Vec<String>>,
}

impl CsvProcessor {
    fn from_file(path: &str) -> Result<Self, CsvError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines().enumerate();

        let headers = match lines.next() {
            Some((_, Ok(line))) => Self::parse_line(&line, 1)?,
            Some((_, Err(e))) => return Err(CsvError::IoError(e)),
            None => return Err(CsvError::InvalidHeader("Empty file".to_string())),
        };

        if headers.is_empty() {
            return Err(CsvError::InvalidHeader("No headers found".to_string()));
        }

        let mut data = Vec::new();
        for (idx, line_result) in lines {
            let line = line_result?;
            let row = Self::parse_line(&line, idx + 2)?;
            
            if row.len() != headers.len() {
                return Err(CsvError::ParseError(
                    format!("Expected {} columns, found {}", headers.len(), row.len()),
                    idx + 2,
                ));
            }
            
            data.push(row);
        }

        Ok(CsvProcessor { headers, data })
    }

    fn parse_line(line: &str, line_number: usize) -> Result<Vec<String>, CsvError> {
        let mut result = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes && chars.peek() == Some(&'"') {
                        current_field.push('"');
                        chars.next();
                    } else {
                        in_quotes = !in_quotes;
                    }
                }
                ',' if !in_quotes => {
                    result.push(current_field.trim().to_string());
                    current_field.clear();
                }
                _ => current_field.push(ch),
            }
        }

        result.push(current_field.trim().to_string());

        if in_quotes {
            return Err(CsvError::ParseError(
                "Unclosed quotation mark".to_string(),
                line_number,
            ));
        }

        Ok(result)
    }

    fn get_column(&self, column_name: &str) -> Result<Vec<&str>, CsvError> {
        let idx = self.headers
            .iter()
            .position(|h| h == column_name)
            .ok_or_else(|| CsvError::InvalidHeader(format!("Column '{}' not found", column_name)))?;

        Ok(self.data
            .iter()
            .map(|row| row[idx].as_str())
            .collect())
    }

    fn validate_numeric_column(&self, column_name: &str) -> Result<Vec<f64>, CsvError> {
        let values = self.get_column(column_name)?;
        let mut numeric_values = Vec::new();

        for (idx, value) in values.iter().enumerate() {
            match value.parse::<f64>() {
                Ok(num) => numeric_values.push(num),
                Err(_) => return Err(CsvError::ParseError(
                    format!("Invalid numeric value '{}'", value),
                    idx + 2,
                )),
            }
        }

        Ok(numeric_values)
    }

    fn summary(&self) -> String {
        format!(
            "CSV Summary:\n  Columns: {}\n  Rows: {}\n  Headers: {:?}",
            self.headers.len(),
            self.data.len(),
            self.headers
        )
    }
}

fn process_csv_file() -> Result<(), CsvError> {
    let processor = CsvProcessor::from_file("data.csv")?;
    
    println!("{}", processor.summary());
    
    match processor.validate_numeric_column("price") {
        Ok(prices) => {
            let avg = prices.iter().sum::<f64>() / prices.len() as f64;
            println!("Average price: {:.2}", avg);
        }
        Err(e) => println!("Price validation failed: {}", e),
    }
    
    Ok(())
}

fn main() {
    if let Err(e) = process_csv_file() {
        eprintln!("Error processing CSV: {}", e);
        std::process::exit(1);
    }
}