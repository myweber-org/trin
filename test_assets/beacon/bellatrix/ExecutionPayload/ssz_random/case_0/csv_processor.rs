
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvProcessor {
    input_path: String,
    output_path: String,
    filter_column: Option<usize>,
    filter_value: Option<String>,
}

impl CsvProcessor {
    pub fn new(input_path: &str, output_path: &str) -> Self {
        CsvProcessor {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            filter_column: None,
            filter_value: None,
        }
    }

    pub fn set_filter(&mut self, column: usize, value: &str) -> &mut Self {
        self.filter_column = Some(column);
        self.filter_value = Some(value.to_string());
        self
    }

    pub fn process(&self) -> Result<usize, Box<dyn Error>> {
        let input_file = File::open(Path::new(&self.input_path))?;
        let reader = BufReader::new(input_file);
        let mut output_file = File::create(Path::new(&self.output_path))?;

        let mut processed_count = 0;

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            if line_num == 0 {
                writeln!(output_file, "{}", line)?;
                continue;
            }

            let should_process = match (self.filter_column, &self.filter_value) {
                (Some(col), Some(val)) => {
                    let columns: Vec<&str> = line.split(',').collect();
                    columns.get(col).map(|&v| v == val).unwrap_or(false)
                }
                _ => true,
            };

            if should_process {
                let transformed_line = self.transform_line(&line)?;
                writeln!(output_file, "{}", transformed_line)?;
                processed_count += 1;
            }
        }

        Ok(processed_count)
    }

    fn transform_line(&self, line: &str) -> Result<String, Box<dyn Error>> {
        let mut columns: Vec<String> = line.split(',').map(|s| s.to_string()).collect();
        
        if columns.len() >= 2 {
            let temp = columns[0].clone();
            columns[0] = columns[1].clone();
            columns[1] = temp;
        }

        Ok(columns.join(","))
    }
}

pub fn process_csv_files() -> Result<(), Box<dyn Error>> {
    let mut processor = CsvProcessor::new("input.csv", "output.csv");
    processor.set_filter(2, "active");
    
    let count = processor.process()?;
    println!("Processed {} records", count);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_csv_processing() {
        let test_input = "id,name,status\n1,alice,active\n2,bob,inactive\n3,charlie,active";
        fs::write("test_input.csv", test_input).unwrap();

        let mut processor = CsvProcessor::new("test_input.csv", "test_output.csv");
        processor.set_filter(2, "active");
        
        let result = processor.process();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        let output = fs::read_to_string("test_output.csv").unwrap();
        assert!(output.contains("alice,1,active"));
        assert!(output.contains("charlie,3,active"));
        assert!(!output.contains("bob"));

        fs::remove_file("test_input.csv").unwrap();
        fs::remove_file("test_output.csv").unwrap();
    }
}