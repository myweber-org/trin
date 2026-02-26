use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    selected_columns: Vec<usize>,
    skip_header: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            selected_columns: Vec::new(),
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
        let mut results = Vec::new();
        let mut line_number = 0;

        for line in reader.lines() {
            let line = line?;
            line_number += 1;

            if self.config.skip_header && line_number == 1 {
                continue;
            }

            let processed_row = self.process_line(&line);
            results.push(processed_row);
        }

        Ok(results)
    }

    fn process_line(&self, line: &str) -> Vec<String> {
        let parts: Vec<&str> = line.split(self.config.delimiter).collect();
        
        if self.config.selected_columns.is_empty() {
            parts.iter().map(|&s| s.to_string()).collect()
        } else {
            self.config.selected_columns
                .iter()
                .filter_map(|&idx| parts.get(idx).map(|&s| s.to_string()))
                .collect()
        }
    }

    pub fn filter_rows<F>(&self, rows: Vec<Vec<String>>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        rows.into_iter()
            .filter(|row| predicate(row))
            .collect()
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

        let config = CsvConfig {
            delimiter: ',',
            selected_columns: vec![0, 2],
            skip_header: true,
        };
        
        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "London"]);
        assert_eq!(result[1], vec!["Bob", "Paris"]);
    }

    #[test]
    fn test_filtering() {
        let rows = vec![
            vec!["apple".to_string(), "10".to_string()],
            vec!["banana".to_string(), "5".to_string()],
            vec!["orange".to_string(), "8".to_string()],
        ];

        let config = CsvConfig::default();
        let processor = CsvProcessor::new(config);
        
        let filtered = processor.filter_rows(rows, |row| {
            row.get(1)
                .and_then(|qty| qty.parse::<i32>().ok())
                .map(|qty| qty > 5)
                .unwrap_or(false)
        });

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|row| row[0] == "apple"));
        assert!(filtered.iter().any(|row| row[0] == "orange"));
    }
}