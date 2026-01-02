use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: usize,
    filter_value: String,
}

impl CsvProcessor {
    pub fn new(input: &str, output: &str, column: usize, value: &str) -> Self {
        CsvProcessor {
            input_path: input.to_string(),
            output_path: output.to_string(),
            filter_column,
            filter_value: value.to_string(),
        }
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;
        let mut processed_count = 0;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let columns: Vec<&str> = line.split(',').collect();
            if columns.len() > self.filter_column {
                if columns[self.filter_column] == self.filter_value {
                    writeln!(output_file, "{}", line)?;
                    processed_count += 1;
                }
            }
        }

        Ok(processed_count)
    }

    pub fn transform_column<F>(&self, transform_fn: F) -> Result<(), Box<dyn Error>>
    where
        F: Fn(&str) -> String,
    {
        let input_file = File::open(&self.input_path)?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(&self.output_path)?;

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            
            if index == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let mut columns: Vec<&str> = line.split(',').collect();
            if !columns.is_empty() {
                let transformed = transform_fn(columns[0]);
                columns[0] = &transformed;
                writeln!(output_file, "{}", columns.join(","))?;
            }
        }

        Ok(())
    }
}

pub fn validate_csv_format(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return false;
    }
    
    let column_count = lines[0].split(',').count();
    
    for line in lines.iter().skip(1) {
        if line.split(',').count() != column_count {
            return false;
        }
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_data = "id,name,status\n1,Alice,active\n2,Bob,inactive\n3,Charlie,active";
        let input_path = "test_input.csv";
        let output_path = "test_output.csv";
        
        fs::write(input_path, test_data).unwrap();
        
        let processor = CsvProcessor::new(input_path, output_path, 2, "active");
        let result = processor.process().unwrap();
        
        assert_eq!(result, 2);
        
        let output = fs::read_to_string(output_path).unwrap();
        assert!(output.contains("Alice"));
        assert!(output.contains("Charlie"));
        assert!(!output.contains("Bob"));
        
        fs::remove_file(input_path).unwrap();
        fs::remove_file(output_path).unwrap();
    }

    #[test]
    fn test_validation() {
        let valid_csv = "a,b,c\n1,2,3\n4,5,6";
        let invalid_csv = "a,b,c\n1,2\n3,4,5";
        
        assert!(validate_csv_format(valid_csv));
        assert!(!validate_csv_format(invalid_csv));
    }
}