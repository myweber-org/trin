use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
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

    pub fn validate_and_clean<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
    ) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(output_path)?;

        let mut line_count = 0;
        let mut valid_rows = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            line_count += 1;

            if index == 0 && self.has_header {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let fields: Vec<&str> = line.split(self.delimiter).collect();
            
            if self.is_valid_row(&fields) {
                writeln!(output_file, "{}", line)?;
                valid_rows += 1;
            } else {
                eprintln!("Warning: Invalid row at line {}: {}", line_count, line);
            }
        }

        println!("Processed {} rows, kept {} valid rows", line_count, valid_rows);
        Ok(valid_rows)
    }

    fn is_valid_row(&self, fields: &[&str]) -> bool {
        if fields.is_empty() {
            return false;
        }

        for field in fields {
            let trimmed = field.trim();
            if trimmed.is_empty() {
                return false;
            }
            
            if trimmed.contains('\n') || trimmed.contains('\r') {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_validation() {
        let input_data = "id,name,value\n1,test,100\n2,,200\n3,data,300\n";
        
        let mut input_file = NamedTempFile::new().unwrap();
        write!(input_file, "{}", input_data).unwrap();
        
        let output_file = NamedTempFile::new().unwrap();
        
        let processor = CsvProcessor::new(',', true);
        let result = processor.validate_and_clean(
            input_file.path(),
            output_file.path()
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
        
        let mut output_content = String::new();
        File::open(output_file.path())
            .unwrap()
            .read_to_string(&mut output_content)
            .unwrap();
        
        assert!(output_content.contains("1,test,100"));
        assert!(!output_content.contains("2,,200"));
        assert!(output_content.contains("3,data,300"));
    }
}