
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    delimiter: char,
    has_headers: bool,
}

impl CsvProcessor {
    pub fn new(delimiter: char, has_headers: bool) -> Self {
        CsvProcessor {
            delimiter,
            has_headers,
        }
    }

    pub fn validate_file(&self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let path = Path::new(file_path);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut line_count = 0;
        let mut column_count: Option<usize> = None;
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            line_count += 1;
            
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if let Some(expected_count) = column_count {
                if columns.len() != expected_count {
                    return Err(format!(
                        "Line {} has {} columns, expected {}",
                        index + 1,
                        columns.len(),
                        expected_count
                    ).into());
                }
            } else {
                column_count = Some(columns.len());
            }
            
            for (col_idx, value) in columns.iter().enumerate() {
                if value.trim().is_empty() {
                    return Err(format!(
                        "Empty value at line {}, column {}",
                        index + 1,
                        col_idx + 1
                    ).into());
                }
            }
        }
        
        if line_count == 0 {
            return Err("File is empty".into());
        }
        
        Ok(line_count)
    }

    pub fn transform_data(
        &self,
        input_path: &str,
        output_path: &str,
        transform_fn: fn(&str) -> String,
    ) -> Result<(), Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        
        let mut output_file = File::create(output_path)?;
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if self.has_headers && index == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }
            
            let transformed_columns: Vec<String> = line
                .split(self.delimiter)
                .map(transform_fn)
                .collect();
            
            let transformed_line = transformed_columns.join(&self.delimiter.to_string());
            writeln!(output_file, "{}", transformed_line)?;
        }
        
        Ok(())
    }

    pub fn filter_rows(
        &self,
        input_path: &str,
        output_path: &str,
        filter_fn: fn(&[&str]) -> bool,
    ) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        
        let mut output_file = File::create(output_path)?;
        let mut kept_rows = 0;
        
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if self.has_headers && index == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }
            
            let columns: Vec<&str> = line.split(self.delimiter).collect();
            
            if filter_fn(&columns) {
                writeln!(output_file, "{}", line)?;
                kept_rows += 1;
            }
        }
        
        Ok(kept_rows)
    }
}

fn uppercase_transform(value: &str) -> String {
    value.to_uppercase()
}

fn numeric_filter(columns: &[&str]) -> bool {
    if columns.len() >= 2 {
        columns[1].parse::<f64>().is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_validation() {
        let processor = CsvProcessor::new(',', false);
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "John,Doe,30").unwrap();
        writeln!(temp_file, "Jane,Smith,25").unwrap();
        
        let result = processor.validate_file(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_data_transformation() {
        let processor = CsvProcessor::new(',', false);
        
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "hello,world").unwrap();
        writeln!(input_file, "test,data").unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        processor.transform_data(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            uppercase_transform,
        ).unwrap();
        
        let mut content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        
        assert_eq!(content, "HELLO,WORLD\nTEST,DATA\n");
    }

    #[test]
    fn test_row_filtering() {
        let processor = CsvProcessor::new(',', false);
        
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "item1,100").unwrap();
        writeln!(input_file, "item2,invalid").unwrap();
        writeln!(input_file, "item3,200").unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let kept = processor.filter_rows(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            numeric_filter,
        ).unwrap();
        
        assert_eq!(kept, 2);
    }
}