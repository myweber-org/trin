
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
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
    pub fn new() -> Self {
        CsvProcessor {
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

    pub fn process_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<Vec<CsvRecord>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_number == 0 && self.has_header {
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let columns: Vec<String> = line
                .split(self.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            if !columns.is_empty() {
                records.push(CsvRecord { columns });
            }
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: Vec<CsvRecord>, predicate: F) -> Vec<CsvRecord>
    where
        F: Fn(&CsvRecord) -> bool,
    {
        records.into_iter().filter(predicate).collect()
    }

    pub fn print_records(&self, records: &[CsvRecord]) {
        for (i, record) in records.iter().enumerate() {
            println!("Record {}: {:?}", i + 1, record.columns);
        }
    }
}

pub fn calculate_column_stats(records: &[CsvRecord], column_index: usize) -> Option<(f64, f64)> {
    if records.is_empty() {
        return None;
    }

    let mut sum = 0.0;
    let mut count = 0;

    for record in records {
        if column_index < record.columns.len() {
            if let Ok(value) = record.columns[column_index].parse::<f64>() {
                sum += value;
                count += 1;
            }
        }
    }

    if count > 0 {
        let average = sum / count as f64;
        Some((sum, average))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();
        writeln!(temp_file, "Charlie,35,Paris").unwrap();

        let processor = CsvProcessor::new();
        let records = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].columns, vec!["Alice", "30", "New York"]);
    }

    #[test]
    fn test_filter_records() {
        let records = vec![
            CsvRecord { columns: vec!["A".to_string(), "10".to_string()] },
            CsvRecord { columns: vec!["B".to_string(), "20".to_string()] },
            CsvRecord { columns: vec!["C".to_string(), "30".to_string()] },
        ];

        let processor = CsvProcessor::new();
        let filtered = processor.filter_records(records, |r| {
            r.columns[1].parse::<i32>().unwrap_or(0) > 15
        });

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_column_stats() {
        let records = vec![
            CsvRecord { columns: vec!["10".to_string()] },
            CsvRecord { columns: vec!["20".to_string()] },
            CsvRecord { columns: vec!["30".to_string()] },
        ];

        let stats = calculate_column_stats(&records, 0).unwrap();
        assert_eq!(stats.0, 60.0);
        assert_eq!(stats.1, 20.0);
    }
}