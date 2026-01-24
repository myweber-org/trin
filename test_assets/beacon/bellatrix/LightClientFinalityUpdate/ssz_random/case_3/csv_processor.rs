use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    pub delimiter: char,
    pub has_headers: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            has_headers: true,
        }
    }
}

pub fn parse_csv<P: AsRef<Path>>(
    path: P,
    config: &CsvConfig,
) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut lines = reader.lines().enumerate();

    if config.has_headers {
        if let Some((_, header_line)) = lines.next() {
            let headers = parse_line(&header_line?, config.delimiter);
            if headers.is_empty() {
                return Err("Empty header row".into());
            }
        }
    }

    for (line_num, line_result) in lines {
        let line = line_result?;
        let fields = parse_line(&line, config.delimiter);
        
        if fields.is_empty() {
            return Err(format!("Empty data row at line {}", line_num + 1).into());
        }
        
        records.push(fields);
    }

    if records.is_empty() {
        return Err("No data records found".into());
    }

    Ok(records)
}

fn parse_line(line: &str, delimiter: char) -> Vec<String> {
    line.split(delimiter)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn validate_records(records: &[Vec<String>], expected_columns: usize) -> Result<(), String> {
    for (i, row) in records.iter().enumerate() {
        if row.len() != expected_columns {
            return Err(format!(
                "Row {} has {} columns, expected {}",
                i + 1,
                row.len(),
                expected_columns
            ));
        }
        
        for (j, field) in row.iter().enumerate() {
            if field.is_empty() {
                return Err(format!("Empty field at row {}, column {}", i + 1, j + 1));
            }
        }
    }
    Ok(())
}