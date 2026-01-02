use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    selected_columns: Option<Vec<usize>>,
    skip_header: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            selected_columns: None,
            skip_header: false,
        }
    }
}

pub struct CsvProcessor {
    config: CsvConfig,
}

impl CsvProcessor {
    pub fn new(config: CsvConfig) -> Self {
        CsvProcessor { config }
    }

    pub fn process_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            let line = line?;
            line_number += 1;

            if self.config.skip_header && line_number == 1 {
                continue;
            }

            let record: Vec<String> = line
                .split(self.config.delimiter)
                .map(|s| s.trim().to_string())
                .collect();

            let processed_record = if let Some(ref selected) = self.config.selected_columns {
                selected
                    .iter()
                    .filter_map(|&idx| record.get(idx).cloned())
                    .collect()
            } else {
                record
            };

            if !processed_record.is_empty() {
                records.push(processed_record);
            }
        }

        Ok(records)
    }

    pub fn filter_records<F>(&self, records: &[Vec<String>], predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        records
            .iter()
            .filter(|record| predicate(record))
            .cloned()
            .collect()
    }
}

pub fn create_config(delimiter: char, columns: Option<Vec<usize>>, skip_header: bool) -> CsvConfig {
    CsvConfig {
        delimiter,
        selected_columns: columns,
        skip_header,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_processing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,London").unwrap();
        writeln!(temp_file, "Bob,25,Paris").unwrap();

        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "30", "London"]);
    }

    #[test]
    fn test_column_selection() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "a,b,c,d").unwrap();
        writeln!(temp_file, "1,2,3,4").unwrap();

        let config = create_config(',', Some(vec![0, 2]), false);
        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result[0], vec!["1", "3"]);
    }

    #[test]
    fn test_filtering() {
        let records = vec![
            vec!["apple".to_string(), "10".to_string()],
            vec!["banana".to_string(), "5".to_string()],
            vec!["orange".to_string(), "8".to_string()],
        ];

        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        let filtered = processor.filter_records(&records, |record| {
            record[1].parse::<i32>().unwrap_or(0) > 7
        });

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|r| r[0] == "apple"));
        assert!(filtered.iter().any(|r| r[0] == "orange"));
    }
}