
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

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

    pub fn validate_file(&self, file_path: &str) -> Result<bool, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        if let Some(first_line) = lines.next() {
            let line = first_line?;
            let fields: Vec<&str> = line.split(self.delimiter).collect();
            
            if fields.len() < 2 {
                return Ok(false);
            }
            
            for field in &fields {
                if field.trim().is_empty() {
                    return Ok(false);
                }
            }
        } else {
            return Ok(false);
        }

        let mut line_count = 1;
        for line_result in lines {
            let line = line_result?;
            line_count += 1;
            
            let fields: Vec<&str> = line.split(self.delimiter).collect();
            if fields.len() < 2 {
                eprintln!("Invalid field count at line {}", line_count);
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn transform_to_uppercase(&self, input_path: &str, output_path: &str) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        let mut processed_lines = 0;
        let mut is_first_line = true;

        for line_result in reader.lines() {
            let line = line_result?;
            
            if is_first_line && self.has_headers {
                writeln!(output_file, "{}", line)?;
                is_first_line = false;
                continue;
            }

            let transformed_line: String = line
                .split(self.delimiter)
                .map(|field| field.to_uppercase())
                .collect::<Vec<String>>()
                .join(&self.delimiter.to_string());

            writeln!(output_file, "{}", transformed_line)?;
            processed_lines += 1;
        }

        Ok(processed_lines)
    }

    pub fn filter_by_column_value(
        &self,
        input_path: &str,
        output_path: &str,
        column_index: usize,
        filter_value: &str,
    ) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        let mut filtered_count = 0;
        let mut is_first_line = true;

        for line_result in reader.lines() {
            let line = line_result?;
            
            if is_first_line && self.has_headers {
                writeln!(output_file, "{}", line)?;
                is_first_line = false;
                continue;
            }

            let fields: Vec<&str> = line.split(self.delimiter).collect();
            
            if column_index < fields.len() && fields[column_index] == filter_value {
                writeln!(output_file, "{}", line)?;
                filtered_count += 1;
            }
        }

        Ok(filtered_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "name,age,city").unwrap();
        writeln!(temp_file, "John,30,New York").unwrap();
        writeln!(temp_file, "Jane,25,London").unwrap();

        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_file(temp_file.path().to_str().unwrap());
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_uppercase_transformation() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "john,30,new york").unwrap();
        writeln!(input_file, "jane,25,london").unwrap();

        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.transform_to_uppercase(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_column_filtering() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "name,age,city").unwrap();
        writeln!(input_file, "John,30,New York").unwrap();
        writeln!(input_file, "Jane,25,London").unwrap();
        writeln!(input_file, "Bob,30,Paris").unwrap();

        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.filter_by_column_value(
            input_file.path().to_str().unwrap(),
            output_file.path().to_str().unwrap(),
            1,
            "30",
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }
}