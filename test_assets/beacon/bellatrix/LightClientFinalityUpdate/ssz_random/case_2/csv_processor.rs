use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    pub delimiter: char,
    pub selected_columns: Vec<usize>,
    pub skip_header: bool,
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

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if self.config.skip_header && line_num == 0 {
                continue;
            }

            let parsed_row = self.parse_line(&line);
            let filtered_row = self.filter_columns(parsed_row);
            
            if !filtered_row.is_empty() {
                results.push(filtered_row);
            }
        }

        Ok(results)
    }

    fn parse_line(&self, line: &str) -> Vec<String> {
        line.split(self.config.delimiter)
            .map(|s| s.trim().to_string())
            .collect()
    }

    fn filter_columns(&self, row: Vec<String>) -> Vec<String> {
        if self.config.selected_columns.is_empty() {
            return row;
        }

        self.config.selected_columns
            .iter()
            .filter_map(|&idx| row.get(idx).cloned())
            .collect()
    }
}

pub fn create_default_config() -> CsvConfig {
    CsvConfig {
        delimiter: ',',
        selected_columns: Vec::new(),
        skip_header: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "Alice,30,New York").unwrap();
        writeln!(temp_file, "Bob,25,London").unwrap();

        let config = CsvConfig {
            delimiter: ',',
            selected_columns: vec![0, 2],
            skip_header: true,
        };

        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "New York"]);
        assert_eq!(result[1], vec!["Bob", "London"]);
    }

    #[test]
    fn test_custom_delimiter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name|age|city").unwrap();
        writeln!(temp_file, "Alice|30|New York").unwrap();

        let config = CsvConfig {
            delimiter: '|',
            selected_columns: vec![1],
            skip_header: true,
        };

        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();

        assert_eq!(result[0], vec!["30"]);
    }
}