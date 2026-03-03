use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct CsvConfig {
    delimiter: char,
    selected_columns: Vec<usize>,
    has_header: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            selected_columns: Vec::new(),
            has_header: true,
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
        let mut lines = reader.lines();

        if self.config.has_header {
            lines.next();
        }

        let mut result = Vec::new();
        for line in lines {
            let line = line?;
            let fields: Vec<String> = line.split(self.config.delimiter).map(String::from).collect();
            
            if self.config.selected_columns.is_empty() {
                result.push(fields);
            } else {
                let selected: Vec<String> = self.config.selected_columns
                    .iter()
                    .filter_map(|&idx| fields.get(idx).cloned())
                    .collect();
                result.push(selected);
            }
        }
        Ok(result)
    }

    pub fn summarize(&self, data: &[Vec<String>]) -> (usize, usize) {
        let row_count = data.len();
        let col_count = data.first().map_or(0, |row| row.len());
        (row_count, col_count)
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

        let config = CsvConfig {
            selected_columns: vec![0, 2],
            ..Default::default()
        };
        let processor = CsvProcessor::new(config);
        let result = processor.process_file(temp_file.path()).unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Alice", "New York"]);
        assert_eq!(result[1], vec!["Bob", "London"]);
        
        let summary = processor.summarize(&result);
        assert_eq!(summary, (2, 2));
    }
}