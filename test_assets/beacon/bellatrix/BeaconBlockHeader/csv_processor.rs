use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub id: u32,
    pub name: String,
    pub value: f64,
    pub active: bool,
}

#[derive(Debug)]
pub enum CsvError {
    IoError(String),
    ParseError(String, usize),
    ValidationError(String),
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvError::IoError(msg) => write!(f, "IO error: {}", msg),
            CsvError::ParseError(msg, line) => write!(f, "Parse error at line {}: {}", line, msg),
            CsvError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for CsvError {}

pub struct CsvProcessor {
    delimiter: char,
    strict_mode: bool,
}

impl Default for CsvProcessor {
    fn default() -> Self {
        CsvProcessor {
            delimiter: ',',
            strict_mode: false,
        }
    }
}

impl CsvProcessor {
    pub fn new(delimiter: char, strict_mode: bool) -> Self {
        CsvProcessor {
            delimiter,
            strict_mode,
        }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, CsvError> {
        let file = File::open(&path).map_err(|e| {
            CsvError::IoError(format!("Failed to open file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line_content = line.map_err(|e| {
                CsvError::IoError(format!("Failed to read line {}: {}", line_number, e))
            })?;

            if line_content.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(&line_content, line_number)?;
            records.push(record);
        }

        if self.strict_mode && records.is_empty() {
            return Err(CsvError::ValidationError(
                "No valid records found in strict mode".to_string(),
            ));
        }

        Ok(records)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let parts: Vec<&str> = line.split(self.delimiter).collect();

        if parts.len() != 4 {
            return Err(CsvError::ParseError(
                format!("Expected 4 fields, found {}", parts.len()),
                line_number,
            ));
        }

        let id = parts[0].parse::<u32>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid ID format: '{}'", parts[0]),
                line_number,
            )
        })?;

        let name = parts[1].trim().to_string();
        if name.is_empty() {
            return Err(CsvError::ParseError(
                "Name cannot be empty".to_string(),
                line_number,
            ));
        }

        let value = parts[2].parse::<f64>().map_err(|_| {
            CsvError::ParseError(
                format!("Invalid value format: '{}'", parts[2]),
                line_number,
            )
        })?;

        let active = match parts[3].trim().to_lowercase().as_str() {
            "true" | "1" | "yes" => true,
            "false" | "0" | "no" => false,
            _ => return Err(CsvError::ParseError(
                format!("Invalid boolean format: '{}'", parts[3]),
                line_number,
            )),
        };

        Ok(CsvRecord {
            id,
            name,
            value,
            active,
        })
    }

    pub fn calculate_statistics(records: &[CsvRecord]) -> (f64, f64, usize) {
        if records.is_empty() {
            return (0.0, 0.0, 0);
        }

        let sum: f64 = records.iter().map(|r| r.value).sum();
        let count = records.len();
        let mean = sum / count as f64;

        let variance: f64 = records
            .iter()
            .map(|r| (r.value - mean).powi(2))
            .sum::<f64>()
            / count as f64;

        let active_count = records.iter().filter(|r| r.active).count();

        (mean, variance.sqrt(), active_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,John Doe,42.5,true").unwrap();
        writeln!(temp_file, "2,Jane Smith,37.8,false").unwrap();
        writeln!(temp_file, "3,Bob Johnson,29.3,yes").unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());

        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].name, "John Doe");
        assert_eq!(records[1].value, 37.8);
        assert!(records[2].active);
    }

    #[test]
    fn test_invalid_csv_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "1,John Doe,invalid,true").unwrap();

        let processor = CsvProcessor::default();
        let result = processor.process_file(temp_file.path());

        assert!(result.is_err());
        if let Err(CsvError::ParseError(msg, line)) = result {
            assert!(msg.contains("Invalid value format"));
            assert_eq!(line, 1);
        }
    }

    #[test]
    fn test_statistics_calculation() {
        let records = vec![
            CsvRecord {
                id: 1,
                name: "Test1".to_string(),
                value: 10.0,
                active: true,
            },
            CsvRecord {
                id: 2,
                name: "Test2".to_string(),
                value: 20.0,
                active: false,
            },
            CsvRecord {
                id: 3,
                name: "Test3".to_string(),
                value: 30.0,
                active: true,
            },
        ];

        let (mean, std_dev, active_count) = CsvProcessor::calculate_statistics(&records);

        assert_eq!(mean, 20.0);
        assert!((std_dev - 8.164965).abs() < 0.0001);
        assert_eq!(active_count, 2);
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
}

pub struct CsvProcessor {
    pub records: Vec<CsvRecord>,
}

impl CsvProcessor {
    pub fn new() -> Self {
        CsvProcessor {
            records: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let columns: Vec<String> = line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            self.records.push(CsvRecord { columns });
        }

        Ok(())
    }

    pub fn filter_by_column_value(&self, column_index: usize, value: &str) -> Vec<&CsvRecord> {
        self.records
            .iter()
            .filter(|record| {
                record.columns.get(column_index)
                    .map(|col| col == value)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn get_column_count(&self) -> Option<usize> {
        self.records.first().map(|record| record.columns.len())
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(processor.records.len(), 2);
    }

    #[test]
    fn test_filter_records() {
        let mut processor = CsvProcessor::new();
        processor.records.push(CsvRecord {
            columns: vec!["Alice".to_string(), "30".to_string(), "New York".to_string()],
        });
        processor.records.push(CsvRecord {
            columns: vec!["Bob".to_string(), "25".to_string(), "London".to_string()],
        });

        let filtered = processor.filter_by_column_value(0, "Alice");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].columns[0], "Alice");
    }

    #[test]
    fn test_empty_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let mut processor = CsvProcessor::new();
        let result = processor.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert!(processor.is_empty());
    }
}use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use csv::{ReaderBuilder, WriterBuilder};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
        }
    }

    pub fn filter_by_column_value(&self, column_name: &str, target_value: &str) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
        
        let output_file = File::create(&self.output_path)?;
        let writer = BufWriter::new(output_file);
        let mut csv_writer = WriterBuilder::new().from_writer(writer);
        
        let headers = csv_reader.headers()?.clone();
        csv_writer.write_record(&headers)?;
        
        let column_index = headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;
        
        for result in csv_reader.records() {
            let record = result?;
            if record.get(column_index) == Some(target_value) {
                csv_writer.write_record(&record)?;
            }
        }
        
        csv_writer.flush()?;
        Ok(())
    }

    pub fn transform_column(&self, column_name: &str, transform_fn: fn(&str) -> String) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut csv_reader = ReaderBuilder::new().has_headers(true).from_reader(reader);
        
        let output_file = File::create(&self.output_path)?;
        let writer = BufWriter::new(output_file);
        let mut csv_writer = WriterBuilder::new().from_writer(writer);
        
        let headers = csv_reader.headers()?.clone();
        csv_writer.write_record(&headers)?;
        
        let column_index = headers.iter()
            .position(|h| h == column_name)
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;
        
        for result in csv_reader.records() {
            let mut record = result?.clone();
            if let Some(value) = record.get(column_index) {
                let transformed = transform_fn(value);
                record[column_index] = transformed.into();
            }
            csv_writer.write_record(&record)?;
        }
        
        csv_writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_filter_by_column_value() {
        let input_data = "name,age,city\nAlice,30,London\nBob,25,Paris\nCharlie,35,London\n";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(&input_file, input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        let processor = CsvProcessor::new(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap());
        
        processor.filter_by_column_value("city", "London").unwrap();
        
        let output = fs::read_to_string(output_file.path()).unwrap();
        let expected = "name,age,city\nAlice,30,London\nCharlie,35,London\n";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_transform_column() {
        let input_data = "name,score\nAlice,85\nBob,92\n";
        let input_file = NamedTempFile::new().unwrap();
        fs::write(&input_file, input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        let processor = CsvProcessor::new(input_file.path().to_str().unwrap(), output_file.path().to_str().unwrap());
        
        fn add_grade(value: &str) -> String {
            let score: i32 = value.parse().unwrap();
            format!("{} ({})", value, if score >= 90 { "A" } else { "B" })
        }
        
        processor.transform_column("score", add_grade).unwrap();
        
        let output = fs::read_to_string(output_file.path()).unwrap();
        let expected = "name,score\nAlice,85 (B)\nBob,92 (A)\n";
        assert_eq!(output, expected);
    }
}