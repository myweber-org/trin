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
        Self {
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
        Self { config }
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

    pub fn filter_rows<F>(&self, data: Vec<Vec<String>>, predicate: F) -> Vec<Vec<String>>
    where
        F: Fn(&[String]) -> bool,
    {
        data.into_iter().filter(|row| predicate(row)).collect()
    }
}

pub fn create_config(delimiter: char, columns: Option<Vec<usize>>, header: bool) -> CsvConfig {
    CsvConfig {
        delimiter,
        selected_columns: columns.unwrap_or_default(),
        has_header: header,
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
        let config = CsvConfig {
            selected_columns: vec![0, 2],
            ..Default::default()
        };
        let processor = CsvProcessor::new(config);
        
        let test_data = vec![
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec!["D".to_string(), "E".to_string(), "F".to_string()],
        ];
        
        let filtered = processor.filter_rows(test_data, |row| row[0] == "A");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], vec!["A".to_string(), "C".to_string()]);
    }
}