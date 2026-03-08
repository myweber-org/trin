
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str, filter_column: usize, filter_value: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column,
            filter_value: filter_value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        
        let output_file = File::create(&self.output_path)?;
        let mut writer = std::io::BufWriter::new(output_file);
        
        let mut processed_count = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                writeln!(writer, "{}", line)?;
                continue;
            }
            
            let columns: Vec<&str> = line.split(',').collect();
            
            if columns.len() > self.filter_column {
                if columns[self.filter_column] == self.filter_value {
                    writeln!(writer, "{}", line)?;
                    processed_count += 1;
                }
            }
        }
        
        Ok(processed_count)
    }
    
    pub fn validate_paths(&self) -> Result<(), Box<dyn Error>> {
        if !Path::new(&self.input_path).exists() {
            return Err(format!("Input file not found: {}", self.input_path).into());
        }
        
        let output_dir = Path::new(&self.output_path).parent();
        if let Some(dir) = output_dir {
            if !dir.exists() {
                return Err(format!("Output directory does not exist: {:?}", dir).into());
            }
        }
        
        Ok(())
    }
}

pub fn process_csv_file(
    input_path: &str,
    output_path: &str,
    filter_column: usize,
    filter_value: &str,
) -> Result<usize, Box<dyn Error>> {
    let processor = CsvProcessor::new(input_path, output_path, filter_column, filter_value);
    processor.validate_paths()?;
    processor.process()
}use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct CsvRecord {
    pub columns: Vec<String>,
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

    pub fn extract_column(&self, records: &[CsvRecord], column_index: usize) -> Vec<String> {
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
        let filtered = processor.filter_records(records, |r| r.columns[0] == "A");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].columns[0], "A");
    }
}