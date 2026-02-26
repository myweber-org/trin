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
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
}

pub struct CsvProcessor {
    pub delimiter: char,
    pub has_header: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_header: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_header,
        }
    }

    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut lines = reader.lines().enumerate();

        if self.has_header {
            lines.next();
        }

        for (line_num, line) in lines {
            let line = line?;
            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if columns.is_empty() {
                continue;
            }

            records.push(CsvRecord { columns });
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: Vec<CsvRecord>, predicate: F) -> Vec<CsvRecord>
    where
        F: Fn(&CsvRecord) -> bool,
    {
        records.into_iter().filter(predicate).collect()
    }

    pub fn get_column_values(&self, records: &[CsvRecord], column_index: usize) -> Vec<String> {
        records
            .iter()
            .filter_map(|record| record.columns.get(column_index).cloned())
            .collect()
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

        let processor = CsvProcessor::new(',', true);
        let records = processor.parse_file(temp_file.path()).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].columns, vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            CsvRecord {
                columns: vec!["A".to_string(), "10".to_string()],
            },
            CsvRecord {
                columns: vec!["B".to_string(), "20".to_string()],
            },
        ];

        let processor = CsvProcessor::new(',', false);
        let filtered = processor.filter_records(records, |r| r.columns[1] == "20");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].columns[0], "B");
    }
}