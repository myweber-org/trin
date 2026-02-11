
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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

    pub fn validate_file<P: AsRef<Path>>(&self, file_path: P) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;

        for (index, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if self.has_header && index == 0 {
                continue;
            }

            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            match column_count {
                Some(expected) => {
                    if columns.len() != expected {
                        return Err(format!(
                            "Line {} has {} columns, expected {}",
                            index + 1,
                            columns.len(),
                            expected
                        ).into());
                    }
                }
                None => {
                    column_count = Some(columns.len());
                }
            }

            for (col_index, value) in columns.iter().enumerate() {
                if value.trim().is_empty() {
                    return Err(format!(
                        "Empty value at line {}, column {}",
                        index + 1,
                        col_index + 1
                    ).into());
                }
            }

            line_count += 1;
        }

        if line_count == 0 {
            return Err("File contains no data rows".into());
        }

        Ok(line_count)
    }

    pub fn transform_column<P: AsRef<Path>>(
        &self,
        file_path: P,
        column_index: usize,
        transformer: fn(&str) -> String,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for (index, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            
            if self.has_header && index == 0 {
                continue;
            }

            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if column_index >= columns.len() {
                return Err(format!(
                    "Column index {} out of bounds for line {} with {} columns",
                    column_index,
                    index + 1,
                    columns.len()
                ).into());
            }

            let transformed = transformer(columns[column_index]);
            results.push(transformed);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_csv(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    #[test]
    fn test_validate_valid_csv() {
        let csv_content = "name,age,city\nJohn,30,NYC\nJane,25,LA\n";
        let file = create_test_csv(csv_content);
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_file(file.path());
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_validate_invalid_column_count() {
        let csv_content = "name,age,city\nJohn,30\nJane,25,LA,extra\n";
        let file = create_test_csv(csv_content);
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_file(file.path());
        
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_column() {
        let csv_content = "name,score\nAlice,85\nBob,92\nCharlie,78\n";
        let file = create_test_csv(csv_content);
        
        let processor = CsvProcessor::new(',', true);
        
        fn add_grade(value: &str) -> String {
            let score: i32 = value.parse().unwrap();
            format!("{} -> {}", value, if score >= 90 { "A" } else { "B" })
        }
        
        let result = processor.transform_column(file.path(), 1, add_grade);
        
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed.len(), 3);
        assert_eq!(transformed[0], "85 -> B");
        assert_eq!(transformed[1], "92 -> A");
    }
}